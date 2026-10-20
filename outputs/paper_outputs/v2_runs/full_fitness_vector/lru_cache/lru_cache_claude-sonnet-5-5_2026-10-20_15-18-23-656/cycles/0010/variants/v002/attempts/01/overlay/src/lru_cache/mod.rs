// Segmented LRU cache using HashMap + arena-based doubly-linked lists.
// Two segments: probationary (new keys) and protected (keys hit at least once).
// Nodes live in a Vec (arena); on eviction the slot is reused in place.
// The segment flag is stored in the high bit of the `prev` link; links are u32.
// The key->index map uses a single-multiply Fx hasher with no avalanche fold;
// finish() rotates so the well-mixed high product bits land in the low bits.
// A ghost fingerprint table remembers recently evicted keys: never-seen keys
// are inserted at the LRU end of probation (LIP-style), ghost hits at MRU.
// Ghost entries carry a 2-bit saturating re-seen counter in spare bits
// (bits 1..2; bit 0 = occupied). A cold newcomer does not evict a probation
// victim whose counter >= 1: the victim is spared (counter decremented, moved
// to probation MRU) and the next probation node is evicted instead.
// Counters are halved each time the ghost table wraps.
// Ghost work is skipped during warm-up (table empty until first eviction).
// No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const FX_K: u64 = 0x517c_c1b7_2722_0a95;

/// Fx-style hasher: one rotate/xor/multiply per word, no xor-shift fold.
#[derive(Default, Clone, Copy)]
pub struct FxHasher {
	h: u64,
}

impl FxHasher {
	#[inline]
	fn add(&mut self, x: u64) {
		self.h = (self.h.rotate_left(5) ^ x).wrapping_mul(FX_K);
	}
}

impl Hasher for FxHasher {
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		for chunk in bytes.chunks(8) {
			let mut buf = [0u8; 8];
			buf[..chunk.len()].copy_from_slice(chunk);
			self.add(u64::from_le_bytes(buf));
		}
	}
	#[inline]
	fn write_u8(&mut self, i: u8) {
		self.add(i as u64);
	}
	#[inline]
	fn write_u32(&mut self, i: u32) {
		self.add(i as u64);
	}
	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.add(i);
	}
	#[inline]
	fn write_i32(&mut self, i: i32) {
		self.add(i as u32 as u64);
	}
	#[inline]
	fn write_i64(&mut self, i: i64) {
		self.add(i as u64);
	}
	#[inline]
	fn write_usize(&mut self, i: usize) {
		self.add(i as u64);
	}
	#[inline]
	fn finish(&self) -> u64 {
		self.h.rotate_left(26)
	}
}

type FxBuild = BuildHasherDefault<FxHasher>;

const PROB: usize = 0;
const PROT: usize = 1;

const SEG_BIT: u32 = 1 << 31;
const IDX_MASK: u32 = SEG_BIT - 1;
const NIL: u32 = IDX_MASK; // "no link" sentinel (also valid in masked form)

/// Ghost entry layout: bit 0 = occupied, bits 1..2 = counter, bits 3.. = fingerprint.
const FP_MASK: u32 = !0b110;

#[inline]
fn ghost_counter(entry: u32, fp: u32) -> u32 {
	if entry & FP_MASK == fp {
		(entry >> 1) & 3
	} else {
		0
	}
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32, // high bit = protected segment flag, low 31 bits = prev index
	next: u32,
}

impl<K, V> Node<K, V> {
	#[inline]
	fn segment(&self) -> bool {
		self.prev & SEG_BIT != 0
	}
	#[inline]
	fn set_segment(&mut self, protected: bool) {
		self.prev = (self.prev & IDX_MASK) | if protected { SEG_BIT } else { 0 };
	}
	#[inline]
	fn prev_idx(&self) -> u32 {
		self.prev & IDX_MASK
	}
	#[inline]
	fn set_prev(&mut self, p: u32) {
		self.prev = (self.prev & SEG_BIT) | (p & IDX_MASK);
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32, FxBuild>, // key -> node index
	nodes: Vec<Node<K, V>>,        // arena
	heads: [u32; 2],               // LRU end per segment
	tails: [u32; 2],               // MRU end per segment
	protected_len: usize,
	protected_cap: usize,
	capacity: usize,
	ghost: Vec<u32>, // fingerprints + counters of evicted keys; 0 = empty
	ghost_mask: usize,
	ghost_writes: usize,
}

impl<K, V> LruCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("LruCache.try_new: capacity must be > 0");
			return None;
		}
		if capacity >= NIL as usize {
			log::error!("LruCache.try_new: capacity too large");
			return None;
		}
		log::info!("LruCache.try_new: capacity={capacity}");
		// Protected segment is 3/4 of capacity; probationary keeps the remainder (>= 1).
		let protected_cap = if capacity > 1 {
			(capacity - capacity / 4).min(capacity - 1).max(1)
		} else {
			0
		};
		let ghost_len = capacity.next_power_of_two().max(2);
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FxBuild::default()),
			nodes: Vec::with_capacity(capacity),
			heads: [NIL, NIL],
			tails: [NIL, NIL],
			protected_len: 0,
			protected_cap,
			capacity,
			ghost: vec![0; ghost_len],
			ghost_mask: ghost_len - 1,
			ghost_writes: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.promote(idx);
		Some(&self.nodes[idx as usize].value)
	}

	#[inline]
	fn ghost_slot(&self, key: &K) -> (usize, u32) {
		let h = self.map.hasher().hash_one(key);
		((h as usize) & self.ghost_mask, (((h >> 32) as u32) | 1) & FP_MASK)
	}

	/// Inserts or updates `key`. If at capacity, evicts the probationary LRU
	/// (or the protected LRU if probationary is empty).
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx as usize].value = value;
			self.promote(idx);
			return;
		}
		if self.nodes.len() >= self.capacity {
			// Ghost lookup: has this key been evicted recently?
			let (gi, gf) = self.ghost_slot(&key);
			let ge = self.ghost[gi];
			let seen = ge & FP_MASK == gf;
			if seen {
				// Re-seen: bump the saturating counter, keep the entry.
				let c = ((ge >> 1) & 3).min(2) + 1;
				self.ghost[gi] = gf | (c << 1);
			}
			let mut victim = if self.heads[PROB] != NIL { self.heads[PROB] } else { self.heads[PROT] };
			assert!(victim != NIL, "non-empty list has a head");
			let (mut ei, mut ef) = self.ghost_slot(&self.nodes[victim as usize].key);
			let mut vc = ghost_counter(self.ghost[ei], ef);
			// Admission: a cold newcomer must not displace a proven victim.
			if !seen && vc >= 1 && self.heads[PROB] == victim && self.nodes[victim as usize].next != NIL {
				self.ghost[ei] = ef | ((vc - 1) << 1);
				let nxt = self.nodes[victim as usize].next;
				self.unlink(victim);
				self.push_back(victim, PROB);
				victim = nxt;
				let s = self.ghost_slot(&self.nodes[victim as usize].key);
				ei = s.0;
				ef = s.1;
				vc = ghost_counter(self.ghost[ei], ef);
			}
			let vi = victim as usize;
			self.map.remove(&self.nodes[vi].key);
			self.ghost[ei] = ef | (vc << 1);
			// Halve counters each time the ghost table wraps.
			self.ghost_writes += 1;
			if self.ghost_writes > self.ghost_mask {
				self.ghost_writes = 0;
				for e in self.ghost.iter_mut() {
					if *e != 0 {
						*e = (*e & FP_MASK) | (((*e >> 2) & 1) << 1);
					}
				}
			}
			self.unlink(victim);
			self.nodes[vi].key = key.clone();
			self.nodes[vi].value = value;
			if seen {
				self.push_back(victim, PROB);
			} else {
				self.push_front(victim, PROB);
			}
			self.map.insert(key, victim);
		} else {
			// Warm-up: ghost table is empty, nothing was ever evicted.
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
			});
			self.push_front(idx, PROB);
			self.map.insert(key, idx);
		}
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Handle a hit on `idx`.
	fn promote(&mut self, idx: u32) {
		if self.nodes[idx as usize].segment() {
			self.unlink(idx);
			self.push_back(idx, PROT);
			return;
		}
		if self.protected_cap == 0 {
			self.unlink(idx);
			self.push_back(idx, PROB);
			return;
		}
		self.unlink(idx);
		self.push_back(idx, PROT);
		if self.protected_len > self.protected_cap {
			let h = self.heads[PROT];
			if h != NIL {
				self.unlink(h);
				self.push_back(h, PROB);
			}
		}
	}

	/// Remove node from its current segment's list.
	fn unlink(&mut self, idx: u32) {
		let i = idx as usize;
		let s = if self.nodes[i].segment() { PROT } else { PROB };
		let prev = self.nodes[i].prev_idx();
		let next = self.nodes[i].next;
		if prev != NIL {
			self.nodes[prev as usize].next = next;
		} else {
			self.heads[s] = next;
		}
		if next != NIL {
			self.nodes[next as usize].set_prev(prev);
		} else {
			self.tails[s] = prev;
		}
		self.nodes[i].set_prev(NIL);
		self.nodes[i].next = NIL;
		if s == PROT {
			self.protected_len -= 1;
		}
	}

	/// Append node to the MRU end of segment `s`.
	fn push_back(&mut self, idx: u32, s: usize) {
		let i = idx as usize;
		let tail = self.tails[s];
		self.nodes[i].set_segment(s == PROT);
		self.nodes[i].set_prev(tail);
		self.nodes[i].next = NIL;
		if tail != NIL {
			self.nodes[tail as usize].next = idx;
		} else {
			self.heads[s] = idx;
		}
		self.tails[s] = idx;
		if s == PROT {
			self.protected_len += 1;
		}
	}

	/// Insert node at the LRU end (head) of segment `s`.
	fn push_front(&mut self, idx: u32, s: usize) {
		let i = idx as usize;
		let head = self.heads[s];
		self.nodes[i].set_segment(s == PROT);
		self.nodes[i].set_prev(NIL);
		self.nodes[i].next = head;
		if head != NIL {
			self.nodes[head as usize].set_prev(idx);
		} else {
			self.tails[s] = idx;
		}
		self.heads[s] = idx;
		if s == PROT {
			self.protected_len += 1;
		}
	}
}

impl<K, V> framework::Cache<K, V> for LruCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	fn put(&mut self, key: K, value: V) {
		LruCache::put(self, key, value);
	}

	fn get(&mut self, key: &K) -> Option<&V> {
		LruCache::get(self, key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_miss_returns_none() {
		let mut cache = LruCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn capacity_one() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
