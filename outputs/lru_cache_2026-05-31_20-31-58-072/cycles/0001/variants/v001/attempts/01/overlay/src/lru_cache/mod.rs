// O(1) SLRU cache using HashMap + arena-based doubly-linked lists.
//
// Two segments share one arena (`Vec<Node<K, V>>`):
//   * `probationary` (small): newly admitted keys. Eviction is always taken
//     from this segment's LRU head, so the resident count never exceeds the
//     overall capacity.
//   * `protected` (large): keys that earned a second reference. A protected
//     hit refreshes recency without disturbing probationary residents, so
//     scan-mixed workloads stop trampling the working set.
//
// Sizing follows the standard SLRU split: protected gets 4/5 of capacity,
// probationary gets the remainder. Each node carries a `segment` tag so the
// list helpers know which set of head/tail pointers to mutate.
//
// Slot reuse on eviction mirrors the baseline: the arena slot of the evicted
// probationary LRU is overwritten in place, no free list needed. No unsafe,
// no extra crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

const SEG_PROBATIONARY: u8 = 0;
const SEG_PROTECTED: u8 = 1;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	segment: u8,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	// Probationary segment endpoints. head = LRU, tail = MRU.
	prob_head: Option<usize>,
	prob_tail: Option<usize>,
	prob_len: usize,
	prob_capacity: usize,
	// Protected segment endpoints. head = LRU, tail = MRU.
	prot_head: Option<usize>,
	prot_tail: Option<usize>,
	prot_len: usize,
	prot_capacity: usize,
	capacity: usize,
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
		// Classic 80/20 SLRU split. With capacity=1 the protected segment
		// collapses to 0 and the cache degenerates to a single-slot LRU on
		// the probationary list, which still satisfies the contract.
		let prot_capacity = capacity * 4 / 5;
		let prob_capacity = capacity - prot_capacity;
		log::info!(
			"LruCache.try_new (SLRU): capacity={capacity} prot_capacity={prot_capacity} prob_capacity={prob_capacity}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			prob_head: None,
			prob_tail: None,
			prob_len: 0,
			prob_capacity,
			prot_head: None,
			prot_tail: None,
			prot_len: 0,
			prot_capacity,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, applying the SLRU promotion
	/// rule: a probationary hit moves to the protected MRU end (demoting the
	/// protected LRU back to probationary if protected overflows); a protected
	/// hit just refreshes its own segment.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.promote(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. New keys land at the MRU end of the
	/// probationary segment; if the probationary segment is at capacity, its
	/// LRU head is evicted first and the slot reused. Updates apply the same
	/// promotion rule as a successful get.
	pub fn put(&mut self, key: K, value: V) {
		// Existing key: update value in place and apply promotion.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.promote(idx);
			return;
		}
		// New key — admit into probationary. If the probationary segment is
		// at capacity, evict its LRU head and reuse the arena slot; otherwise
		// allocate a fresh slot.
		let idx = if self.prob_len >= self.prob_capacity {
			debug_assert!(self.prob_head.is_some(), "probationary head must be Some when at capacity");
			let head_idx = self.prob_head.expect("non-empty probationary has a head");
			let evicted_key = self.nodes[head_idx].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(head_idx);
			self.nodes[head_idx].key = key.clone();
			self.nodes[head_idx].value = value;
			head_idx
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				segment: SEG_PROBATIONARY,
			});
			idx
		};
		self.push_to_segment(idx, SEG_PROBATIONARY);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert!(self.prob_len <= self.prob_capacity);
		debug_assert!(self.prot_len <= self.prot_capacity);
	}

	/// Apply the SLRU promotion rule to the node at `idx`.
	///
	/// * Protected hit: move to MRU end of protected.
	/// * Probationary hit: move to MRU end of protected. If that overflows
	///   the protected segment, demote its LRU head back to the MRU end of
	///   probationary (only the segment tag and list membership change; the
	///   key/value stay in place).
	///
	/// In the degenerate `prot_capacity == 0` case (only possible at
	/// capacity=1), a probationary hit just refreshes recency in
	/// probationary.
	fn promote(&mut self, idx: usize) {
		let segment = self.nodes[idx].segment;
		if segment == SEG_PROTECTED {
			self.unlink(idx);
			self.push_to_segment(idx, SEG_PROTECTED);
			return;
		}
		// Probationary hit.
		self.unlink(idx);
		if self.prot_capacity == 0 {
			// No protected segment available — keep the entry in probationary.
			self.push_to_segment(idx, SEG_PROBATIONARY);
			return;
		}
		self.push_to_segment(idx, SEG_PROTECTED);
		if self.prot_len > self.prot_capacity {
			let demote_idx = self.prot_head.expect("protected non-empty after overflow");
			self.unlink(demote_idx);
			self.push_to_segment(demote_idx, SEG_PROBATIONARY);
		}
	}

	/// Detach `idx` from whichever segment it currently belongs to. Updates
	/// that segment's head, tail, and length. The node's `segment` tag is
	/// left untouched so callers can read it before deciding where to
	/// re-attach.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let segment = self.nodes[idx].segment;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => match segment {
				SEG_PROBATIONARY => self.prob_head = next,
				SEG_PROTECTED => self.prot_head = next,
				_ => unreachable!("unknown segment tag {segment}"),
			},
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => match segment {
				SEG_PROBATIONARY => self.prob_tail = prev,
				SEG_PROTECTED => self.prot_tail = prev,
				_ => unreachable!("unknown segment tag {segment}"),
			},
		}
		match segment {
			SEG_PROBATIONARY => {
				debug_assert!(self.prob_len > 0, "unlink underflow on probationary");
				self.prob_len -= 1;
			}
			SEG_PROTECTED => {
				debug_assert!(self.prot_len > 0, "unlink underflow on protected");
				self.prot_len -= 1;
			}
			_ => unreachable!("unknown segment tag {segment}"),
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append `idx` at the MRU (tail) end of `segment`. Sets the node's
	/// `segment` tag, fixes prev/next pointers, and bumps that segment's
	/// length and tail (and head, if the list was empty).
	fn push_to_segment(&mut self, idx: usize, segment: u8) {
		self.nodes[idx].segment = segment;
		self.nodes[idx].next = None;
		match segment {
			SEG_PROBATIONARY => {
				self.nodes[idx].prev = self.prob_tail;
				match self.prob_tail {
					Some(t) => self.nodes[t].next = Some(idx),
					None => self.prob_head = Some(idx),
				}
				self.prob_tail = Some(idx);
				self.prob_len += 1;
			}
			SEG_PROTECTED => {
				self.nodes[idx].prev = self.prot_tail;
				match self.prot_tail {
					Some(t) => self.nodes[t].next = Some(idx),
					None => self.prot_head = Some(idx),
				}
				self.prot_tail = Some(idx);
				self.prot_len += 1;
			}
			_ => unreachable!("unknown segment tag {segment}"),
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
		let mut cache = LruCache::<i64, i64>::new(8);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get_roundtrip() {
		let mut cache = LruCache::new(8);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn put_overwrites_existing_key() {
		let mut cache = LruCache::new(8);
		cache.put(1, 10);
		cache.put(1, 20);
		assert_eq!(*cache.get(&1).unwrap(), 20);
	}

	#[test]
	fn probationary_hit_promotes_to_protected() {
		// At capacity=8, prot_cap=6, prob_cap=2. Insert two distinct keys;
		// they land in probationary. A second reference promotes one of
		// them into protected, leaving probationary half-empty.
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		cache.put(1, 10);
		cache.put(2, 20);
		// touch key 1: promoted from probationary to protected
		assert_eq!(*cache.get(&1).unwrap(), 10);
		// fill probationary up again to force eviction; key 1 (protected)
		// must survive because eviction takes from probationary.
		for k in 3..20i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k * 10);
			}
		}
		assert_eq!(*cache.get(&1).unwrap(), 10, "promoted key was evicted");
	}

	#[test]
	fn never_put_key_returns_none() {
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for k in 0..20i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		assert!(cache.get(&9999).is_none());
	}

	#[test]
	fn resident_count_stays_within_capacity() {
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for k in 0..200i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let resident = (0..200i64).filter(|k| cache.get(k).is_some()).count();
		assert!(resident <= 8, "resident={resident} exceeds capacity=8");
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
