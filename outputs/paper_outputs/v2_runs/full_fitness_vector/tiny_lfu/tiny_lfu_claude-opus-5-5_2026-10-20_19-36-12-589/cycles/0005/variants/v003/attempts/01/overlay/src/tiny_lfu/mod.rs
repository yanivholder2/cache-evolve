use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// Sentinel for "no link" in the u32-indexed intrusive lists. It fits in 31 bits so the top
/// bit of `Node.next` is free to carry the region tag.
const NIL: u32 = 0x7FFF_FFFF;

/// Top bit of `Node.next`: 1 = node lives in the admission window, 0 = node lives in main.
const WIN_BIT: u32 = 0x8000_0000;

/// Window size as a fraction of capacity: `max(1, capacity / WINDOW_DIVISOR)` (~1%).
const WINDOW_DIVISOR: usize = 100;

const FX_SEED: u64 = 0x517c_c1b7_2722_0a95;

/// Minimal FxHash-style hasher (local; no external crates).
#[derive(Default, Clone, Copy)]
pub struct FxLikeHasher {
	state: u64,
}

impl FxLikeHasher {
	#[inline(always)]
	fn add(&mut self, word: u64) {
		self.state = (self.state.rotate_left(5) ^ word).wrapping_mul(FX_SEED);
	}
}

impl Hasher for FxLikeHasher {
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut chunks = bytes.chunks_exact(8);
		for c in &mut chunks {
			let mut buf = [0u8; 8];
			buf.copy_from_slice(c);
			self.add(u64::from_le_bytes(buf));
		}
		let rem = chunks.remainder();
		if !rem.is_empty() {
			let mut buf = [0u8; 8];
			buf[..rem.len()].copy_from_slice(rem);
			self.add(u64::from_le_bytes(buf));
		}
	}

	#[inline]
	fn write_u8(&mut self, i: u8) {
		self.add(i as u64);
	}

	#[inline]
	fn write_u16(&mut self, i: u16) {
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
	fn write_usize(&mut self, i: usize) {
		self.add(i as u64);
	}

	#[inline]
	fn finish(&self) -> u64 {
		self.state
	}
}

type FxBuild = BuildHasherDefault<FxLikeHasher>;

/// murmur3 fmix64 finaliser for avalanche.
#[inline(always)]
fn fmix64(mut h: u64) -> u64 {
	h ^= h >> 33;
	h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
	h ^= h >> 33;
	h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
	h ^= h >> 33;
	h
}

/// Hash a key once with the fast hasher plus a finaliser mix.
#[inline]
fn key_hash<K: Hash>(key: &K) -> u64 {
	let mut hasher = FxLikeHasher::default();
	key.hash(&mut hasher);
	fmix64(hasher.finish())
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32,
	/// Low 31 bits: next link (or `NIL`). Top bit: region flag (`WIN_BIT`).
	next: u32,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, u32, FxBuild>,
	nodes: Vec<Node<K, V>>,
	win_head: u32,
	win_tail: u32,
	main_head: u32,
	main_tail: u32,
	win_len: usize,
	win_cap: usize,
	capacity: usize,
	sketch: CountMinSketch,
}

impl<K, V> TinyLfuCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("TinyLfuCache.try_new: capacity must be > 0");
			return None;
		}
		if capacity >= NIL as usize {
			log::error!(
				"TinyLfuCache.try_new: capacity={capacity} must be < {NIL} for 31-bit node indices"
			);
			return None;
		}
		let width = (capacity * SKETCH_WIDTH_FACTOR)
			.max(MIN_SKETCH_WIDTH)
			.next_power_of_two();
		let win_cap = (capacity / WINDOW_DIVISOR).max(1);
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} window={win_cap} sketch_width={width}"
		);
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FxBuild::default()),
			nodes: Vec::with_capacity(capacity),
			win_head: NIL,
			win_tail: NIL,
			main_head: NIL,
			main_tail: NIL,
			win_len: 0,
			win_cap,
			capacity,
			sketch: CountMinSketch::new(width),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `depth * width * size_of::<u32>()`.
	///
	/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is
	/// sized at construction from capacity (see `try_new`) and does not grow with population.
	/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		// `get` never uses the post-increment estimate, so use the non-returning variant.
		self.sketch.increment_only(key_hash(key));
		let &idx = self.map.get(key)?;
		// Hit: move to the tail of the node's own list (no window -> main promotion).
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx as usize].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let h = key_hash(&key);
		// The incoming key's estimate is no longer needed: the duel now happens on window
		// overflow, between the window head and the main head.
		self.sketch.increment_only(h);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx as usize].value = value;
			self.unlink(idx);
			self.push_back(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL | WIN_BIT,
			});
			self.push_back(idx);
			self.map.insert(key, idx);
			if self.win_len > self.win_cap {
				// Not full before this insert, so main has room (< capacity - win_cap entries):
				// the window overflow moves straight to main without a duel.
				let cand = self.win_head;
				debug_assert!(cand != NIL);
				self.unlink(cand);
				self.set_window(cand, false);
				self.push_back(cand);
			}
			return;
		}

		// Full: evict before inserting so `nodes.len()` never exceeds `capacity`; the freed
		// slot is reused for the new key, which always enters the window tail.
		let slot = self.make_room();
		let i = slot as usize;
		self.nodes[i].key = key.clone();
		self.nodes[i].value = value;
		self.set_window(slot, true);
		self.push_back(slot);
		self.map.insert(key, slot);
	}

	/// Free one slot in a full cache and return it (unlinked, removed from the map).
	///
	/// If the window is at capacity, the new key would overflow it, so the window head is the
	/// candidate and duels the main head (strict `>`): the winner stays (candidate moves to
	/// main tail), the loser's slot is freed.
	fn make_room(&mut self) -> u32 {
		if self.win_len >= self.win_cap && self.win_head != NIL {
			let cand = self.win_head;
			let victim = self.main_head;
			if victim == NIL {
				// No main region (e.g. capacity 1): window behaves as plain LRU.
				self.evict(cand);
				return cand;
			}
			let cand_freq = self
				.sketch
				.estimate_hash(key_hash(&self.nodes[cand as usize].key));
			let victim_freq = self
				.sketch
				.estimate_hash(key_hash(&self.nodes[victim as usize].key));
			if cand_freq > victim_freq {
				self.evict(victim);
				self.unlink(cand);
				self.set_window(cand, false);
				self.push_back(cand);
				victim
			} else {
				self.evict(cand);
				cand
			}
		} else {
			// Defensive: window below its cap while full means main holds the surplus.
			let victim = self.main_head;
			debug_assert!(victim != NIL, "TinyLfuCache: full cache has no victim");
			self.evict(victim);
			victim
		}
	}

	#[inline]
	fn evict(&mut self, idx: u32) {
		self.unlink(idx);
		self.map.remove(&self.nodes[idx as usize].key);
	}

	#[inline(always)]
	fn link_next(&self, i: usize) -> u32 {
		self.nodes[i].next & !WIN_BIT
	}

	#[inline(always)]
	fn set_next(&mut self, i: usize, v: u32) {
		let n = &mut self.nodes[i].next;
		*n = (*n & WIN_BIT) | v;
	}

	#[inline(always)]
	fn is_window(&self, idx: u32) -> bool {
		self.nodes[idx as usize].next & WIN_BIT != 0
	}

	#[inline(always)]
	fn set_window(&mut self, idx: u32, win: bool) {
		let n = &mut self.nodes[idx as usize].next;
		*n = (*n & !WIN_BIT) | if win { WIN_BIT } else { 0 };
	}

	/// Unlink `idx` from whichever list its region flag selects. The flag is preserved.
	fn unlink(&mut self, idx: u32) {
		let i = idx as usize;
		let win = self.is_window(idx);
		let prev = self.nodes[i].prev;
		let next = self.link_next(i);
		if prev != NIL {
			self.set_next(prev as usize, next);
		} else if win {
			self.win_head = next;
		} else {
			self.main_head = next;
		}
		if next != NIL {
			self.nodes[next as usize].prev = prev;
		} else if win {
			self.win_tail = prev;
		} else {
			self.main_tail = prev;
		}
		self.nodes[i].prev = NIL;
		self.set_next(i, NIL);
		if win {
			self.win_len -= 1;
		}
	}

	/// Append `idx` to the tail of the list its region flag selects.
	fn push_back(&mut self, idx: u32) {
		let i = idx as usize;
		let win = self.is_window(idx);
		let tail = if win { self.win_tail } else { self.main_tail };
		self.nodes[i].prev = tail;
		self.set_next(i, NIL);
		if tail != NIL {
			self.set_next(tail as usize, idx);
		} else if win {
			self.win_head = idx;
		} else {
			self.main_head = idx;
		}
		if win {
			self.win_tail = idx;
			self.win_len += 1;
		} else {
			self.main_tail = idx;
		}
	}
}

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	/// Flattened `SKETCH_DEPTH * width` counters; row `r` occupies `[r * width, (r + 1) * width)`.
	counts: Vec<u32>,
}

// The hand-unrolled paths below assume exactly 4 rows.
const _: () = assert!(SKETCH_DEPTH == 4);

impl CountMinSketch {
	fn new(width: usize) -> Self {
		debug_assert!(width.is_power_of_two());
		let counts = vec![0; SKETCH_DEPTH * width];
		Self { width, counts }
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u32>()`.
	fn heap_bytes(&self) -> usize {
		SKETCH_DEPTH * self.width * std::mem::size_of::<u32>()
	}

	/// Flat index for `row`, derived by double hashing from one 64-bit hash.
	#[inline(always)]
	fn index(&self, h1: u32, h2: u32, row: usize) -> usize {
		let col = (h1.wrapping_add((row as u32).wrapping_mul(h2)) as usize) & (self.width - 1);
		row * self.width + col
	}

	/// The 4 flat indices for hash `h`, one per row.
	#[inline(always)]
	fn indices(&self, h: u64) -> [usize; SKETCH_DEPTH] {
		let h1 = h as u32;
		let h2 = ((h >> 32) as u32) | 1;
		let i0 = self.index(h1, h2, 0);
		let i1 = self.index(h1, h2, 1);
		let i2 = self.index(h1, h2, 2);
		let i3 = self.index(h1, h2, 3);
		debug_assert!(i0 < self.counts.len());
		debug_assert!(i1 < self.counts.len());
		debug_assert!(i2 < self.counts.len());
		debug_assert!(i3 < self.counts.len());
		[i0, i1, i2, i3]
	}

	/// Conservative update, fused into one register-held pass: each counter is loaded once,
	/// the minimum is taken branch-free, and only counters equal to the minimum are bumped
	/// (saturating at `MAX_COUNT`). Returns the key's new estimate (`min + inc`).
	///
	/// Writing back from the locals (instead of reloading, as v003 did) is equivalent because
	/// the 4 indices are always distinct: index `r` lies in row `r`'s range
	/// `[r * width, (r + 1) * width)`, and those ranges are disjoint. So no write can alias a
	/// counter another row's write depends on.
	#[inline(always)]
	#[allow(dead_code)]
	fn increment_hash(&mut self, h: u64) -> u32 {
		let [i0, i1, i2, i3] = self.indices(h);
		// SAFETY: `index` masks the column with `width - 1` and adds `row * width` with
		// `row < SKETCH_DEPTH`, so every index is `< SKETCH_DEPTH * width == counts.len()`
		// (checked by `debug_assert!` in `indices`).
		unsafe {
			let c = &mut self.counts;
			let c0 = *c.get_unchecked(i0);
			let c1 = *c.get_unchecked(i1);
			let c2 = *c.get_unchecked(i2);
			let c3 = *c.get_unchecked(i3);
			let min = c0.min(c1).min(c2.min(c3));
			// 1 unless the minimum is saturated; guards the `c == min` bump against overflow.
			let inc = (min < MAX_COUNT) as u32;
			*c.get_unchecked_mut(i0) = c0 + (((c0 == min) as u32) & inc);
			*c.get_unchecked_mut(i1) = c1 + (((c1 == min) as u32) & inc);
			*c.get_unchecked_mut(i2) = c2 + (((c2 == min) as u32) & inc);
			*c.get_unchecked_mut(i3) = c3 + (((c3 == min) as u32) & inc);
			min + inc
		}
	}

	/// Same conservative update as `increment_hash`, but returns nothing so callers that
	/// ignore the estimate don't keep it live.
	#[inline(always)]
	fn increment_only(&mut self, h: u64) {
		let [i0, i1, i2, i3] = self.indices(h);
		// SAFETY: see `increment_hash`.
		unsafe {
			let c = &mut self.counts;
			let c0 = *c.get_unchecked(i0);
			let c1 = *c.get_unchecked(i1);
			let c2 = *c.get_unchecked(i2);
			let c3 = *c.get_unchecked(i3);
			let min = c0.min(c1).min(c2.min(c3));
			let inc = (min < MAX_COUNT) as u32;
			*c.get_unchecked_mut(i0) = c0 + (((c0 == min) as u32) & inc);
			*c.get_unchecked_mut(i1) = c1 + (((c1 == min) as u32) & inc);
			*c.get_unchecked_mut(i2) = c2 + (((c2 == min) as u32) & inc);
			*c.get_unchecked_mut(i3) = c3 + (((c3 == min) as u32) & inc);
		}
	}

	#[inline]
	fn estimate_hash(&self, h: u64) -> u32 {
		let [i0, i1, i2, i3] = self.indices(h);
		// SAFETY: see `increment_hash`.
		unsafe {
			let c = &self.counts;
			let c0 = *c.get_unchecked(i0);
			let c1 = *c.get_unchecked(i1);
			let c2 = *c.get_unchecked(i2);
			let c3 = *c.get_unchecked(i3);
			c0.min(c1).min(c2.min(c3))
		}
	}
}

impl<K, V> framework::Cache<K, V> for TinyLfuCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	fn put(&mut self, key: K, value: V) {
		TinyLfuCache::put(self, key, value);
	}

	fn get(&mut self, key: &K) -> Option<&V> {
		TinyLfuCache::get(self, key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_miss_returns_none() {
		let mut cache = TinyLfuCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get() {
		let mut cache = TinyLfuCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn node_is_compact() {
		assert_eq!(std::mem::size_of::<Node<u64, u64>>(), 24);
	}

	#[test]
	fn conservative_update_bumps_only_minimum_counters() {
		let mut s = CountMinSketch::new(16);
		let h = key_hash(&7u64);
		let h1 = h as u32;
		let h2 = ((h >> 32) as u32) | 1;
		// Inflate one row's counter so it sits above the others.
		let i0 = s.index(h1, h2, 0);
		s.counts[i0] = 5;
		assert_eq!(s.increment_hash(h), 1);
		assert_eq!(s.counts[i0], 5);
		assert_eq!(s.estimate_hash(h), 1);
		assert_eq!(s.increment_hash(h), 2);
		assert_eq!(s.counts[i0], 5);
	}

	#[test]
	fn conservative_update_saturates() {
		let mut s = CountMinSketch::new(16);
		for c in s.counts.iter_mut() {
			*c = MAX_COUNT;
		}
		let h = key_hash(&3u64);
		assert_eq!(s.increment_hash(h), MAX_COUNT);
		assert!(s.counts.iter().all(|&c| c == MAX_COUNT));
		s.increment_only(h);
		assert!(s.counts.iter().all(|&c| c == MAX_COUNT));
	}

	#[test]
	fn increment_only_matches_increment_hash() {
		let mut a = CountMinSketch::new(16);
		let mut b = CountMinSketch::new(16);
		// Small width + many keys forces plenty of collisions across rows.
		for i in 0..2000u64 {
			let h = key_hash(&(i.wrapping_mul(2654435761) % 97));
			let ra = a.increment_hash(h);
			b.increment_only(h);
			assert_eq!(a.counts, b.counts);
			assert_eq!(ra, b.estimate_hash(h));
		}
	}

	#[test]
	fn eviction_prefers_hot_items() {
		// capacity 2 => win_cap 1, main 1. After puts: window [2], main [1].
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // make key 1 hotter than key 2
		cache.get(&3);
		cache.get(&3);
		// Window overflow: candidate 2 (freq 1) loses to main head 1 (freq 2); 3 enters window.
		cache.put(3, 30);

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn low_frequency_new_item_is_rejected() {
		// Under window semantics a brand-new key is always admitted to the window, so the
		// v001 assertion "put(3) is rejected immediately" cannot hold. Rejection now happens
		// on window overflow: a cold key is rejected when it is pushed out of the window and
		// loses the duel against the main head.
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20); // window [2], main [1]
		cache.get(&1);
		cache.put(3, 30); // candidate 2 (freq 1) loses to 1 (freq 2); window [3]
		cache.put(4, 40); // candidate 3 (freq 1) loses to 1 (freq 3); window [4]

		assert!(cache.get(&3).is_none());
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn update_promotes_existing_item_and_admits_hot_arrivals() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11);
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30);

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn key_put_twice_in_window_stays_resident() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20); // window [2], main [1]
		cache.put(3, 30); // 3 enters window
		cache.put(3, 31); // update while in window
		let idx = cache.map[&3];
		assert!(cache.is_window(idx));
		assert_eq!(cache.win_len, 1);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(cache.map.len(), 2);
	}

	#[test]
	fn window_and_main_respect_capacity() {
		let cap = 300;
		let mut cache = TinyLfuCache::new(cap);
		for k in 0..5000i64 {
			let key = (k * 7919) % 1000;
			if cache.get(&key).is_none() {
				cache.put(key, key);
			}
			assert!(cache.nodes.len() <= cap);
			assert_eq!(cache.map.len(), cache.nodes.len());
			assert!(cache.win_len <= cache.win_cap);
		}
		assert_eq!(cache.win_cap, 3);
		assert_eq!(cache.win_len, 3);
	}

	#[test]
	fn capacity_one_is_plain_lru() {
		let mut cache = TinyLfuCache::new(1);
		cache.put(1, 10);
		cache.put(2, 20);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}
}
