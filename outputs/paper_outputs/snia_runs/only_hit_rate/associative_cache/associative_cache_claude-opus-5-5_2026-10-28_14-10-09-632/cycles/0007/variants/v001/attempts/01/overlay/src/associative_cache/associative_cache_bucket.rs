use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::{debug, trace};

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Saturation ceiling for the GCLOCK reference counter (3 bits of state).
const GCLOCK_MAX: usize = 7;

/// Absolute floor on probation-ring depth, capped at half the bucket.
const PROBATION_FLOOR: usize = 64;

/// Probation ring size for a bucket of `max_size` slots.
///
/// `0` (plain GCLOCK) below 4 slots; otherwise `max(max_size / 4,
/// min(PROBATION_FLOOR, max_size / 2))`. Always `< max_size`, so the main
/// region is non-empty once the bucket is full.
#[inline]
fn probation_len(max_size: usize) -> usize {
	if max_size < 4 {
		0
	} else {
		(max_size / 4).max(PROBATION_FLOOR.min(max_size / 2)).max(1)
	}
}

/// Bucket using an in-place GCLOCK scheme fronted by an S3-FIFO-style
/// probation ring, with a per-slot recent-hit bit gating multiplicative decay.
///
/// `Item::lfu_counter` is reused as a saturating reference counter in
/// `0..=GCLOCK_MAX`. Hits increment it and set the slot's `recent` bit; items
/// never move on a hit.
///
/// Once full, slots `[0, small_len)` form a probation FIFO walked by
/// `small_hand`, and slots `[small_len, len)` form the GCLOCK main region
/// walked by `hand`. On a full-bucket miss the probation slot at `small_hand`
/// is examined: if its counter is 0 it is evicted and replaced by the new item;
/// otherwise it is promoted into main (displacing the sweep's victim, with
/// counter and recent bit cleared) and the new item takes its probation slot.
/// When `max_size < 4`, `small_len == 0` and the bucket is plain GCLOCK.
///
/// Sweep rule at each visited slot: a set recent bit is cleared (counter kept);
/// otherwise a non-zero counter is halved; otherwise the slot is the victim.
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// Recent-hit bit per slot, index-aligned with `items`.
	recent: Vec<bool>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Main-region CLOCK hand. Invariant: `hand < max(items.len(), 1)`.
	hand: usize,
	/// Probation FIFO size (0 disables the probation ring).
	small_len: usize,
	/// Probation FIFO hand. Invariant: `small_hand < max(min(small_len, len), 1)`.
	small_hand: usize,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let max_size = settings.bucket_size;
		let small_len = probation_len(max_size);
		Self {
			items: Vec::with_capacity(max_size),
			recent: Vec::with_capacity(max_size),
			max_size,
			eviction_policy: settings.eviction_policy.build(),
			hand: 0,
			small_len,
			small_hand: 0,
		}
	}

	/// Whether the probation/main split is active (bucket full, ring enabled).
	#[inline]
	fn split_active(&self) -> bool {
		self.small_len > 0 && self.items.len() == self.max_size && self.items.len() > self.small_len
	}

	/// Main-region hand clamped into `[small_len, len)`.
	#[inline]
	fn main_hand(&self) -> usize {
		let len = self.items.len();
		if self.hand < self.small_len || self.hand >= len {
			self.small_len
		} else {
			self.hand
		}
	}

	/// Number of sweep visits slot `i` survives before becoming the victim:
	/// the recent bit plus the bit length of the counter.
	#[inline]
	fn steps(&self, i: usize) -> usize {
		let c = self.items[i].lfu_counter;
		self.recent[i] as usize + (usize::BITS - c.leading_zeros()) as usize
	}

	/// One sweep visit at slot `h`. Returns `true` if `h` is the victim.
	#[inline]
	fn sweep_visit(&mut self, h: usize) -> bool {
		if self.recent[h] {
			self.recent[h] = false;
			false
		} else if self.items[h].lfu_counter > 0 {
			self.items[h].lfu_counter >>= 1;
			false
		} else {
			true
		}
	}

	/// Min-steps-from-hand scan over `[lo, len)` starting at `start`.
	///
	/// Returns the first minimum at or after `start` (wrapping), which is
	/// exactly the sweep's victim: every visit lowers a slot's `steps` by one,
	/// so the slot with the lowest `steps` (earliest from the hand on ties)
	/// reaches 0 first.
	fn scan_min_from(&self, lo: usize, start: usize) -> usize {
		let len = self.items.len();
		let span = len - lo;
		let mut best = start;
		let mut best_steps = self.steps(start);
		if best_steps == 0 {
			return best;
		}
		for step in 1..span {
			let idx = lo + (start - lo + step) % span;
			let s = self.steps(idx);
			if s < best_steps {
				best = idx;
				best_steps = s;
				if s == 0 {
					break;
				}
			}
		}
		best
	}
}

#[inline]
fn gclock_bump(counter: usize) -> usize {
	counter.saturating_add(1).min(GCLOCK_MAX)
}

impl<K, V> Bucket<K, V> for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn get_items(&self) -> &Vec<Item<K, V>> {
		&self.items
	}

	fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>> {
		&mut self.items
	}

	fn get_max_size(&self) -> usize {
		self.max_size
	}

	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	/// GCLOCK hit: saturating counter increment plus recent bit, no reordering.
	#[inline]
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let item = &mut self.items[position];
		item.lfu_counter = gclock_bump(item.lfu_counter);
		self.recent[position] = true;
		&self.items[position]
	}

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Existing key: overwrite value, bump counter, set recent, no movement.
		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			let slot = &mut self.items[position];
			slot.value = item.value;
			slot.lfu_counter = gclock_bump(slot.lfu_counter);
			self.recent[position] = true;
			return (&self.items[position], None);
		}

		let mut item = item;
		item.lfu_counter = 0;

		let len = self.items.len();
		if len < self.max_size {
			self.items.push(item);
			self.recent.push(false);
			let pushed = self.items.last().expect("bucket invariant: just pushed");
			return (pushed, None);
		}

		// Full (len == max_size). max_size == 0 is rejected by settings
		// validation, so len > 0 here.
		debug_assert!(len > 0, "GCLOCK put on zero-capacity bucket");

		if self.split_active() {
			let small_len = self.small_len;
			let p = if self.small_hand < small_len { self.small_hand } else { 0 };
			self.small_hand = (p + 1) % small_len;

			if self.items[p].lfu_counter == 0 {
				// Never hit during probation: evict it, new item takes its slot.
				let evicted = std::mem::replace(&mut self.items[p], item);
				self.recent[p] = false;
				debug!("Bucket.put: policy=S3GCLOCK probation_evicted_key={:?}", evicted.key);
				return (&self.items[p], Some(evicted));
			}

			// Hit during probation: promote into main, displacing the sweep's
			// victim; the new item takes the probation slot.
			let mut hand = self.main_hand();
			while !self.sweep_visit(hand) {
				hand += 1;
				if hand == len {
					hand = small_len;
				}
			}
			let m = hand;
			let mut promoted = std::mem::replace(&mut self.items[p], item);
			self.recent[p] = false;
			promoted.lfu_counter = 0;
			let evicted = std::mem::replace(&mut self.items[m], promoted);
			self.recent[m] = false;
			hand += 1;
			if hand == len {
				hand = small_len;
			}
			self.hand = hand;
			debug!("Bucket.put: policy=S3GCLOCK main_evicted_key={:?}", evicted.key);
			return (&self.items[p], Some(evicted));
		}

		// Plain GCLOCK (small_len == 0).
		let mut hand = if self.hand < len { self.hand } else { 0 };
		while !self.sweep_visit(hand) {
			hand += 1;
			if hand == len {
				hand = 0;
			}
		}
		let victim_idx = hand;
		let evicted = std::mem::replace(&mut self.items[victim_idx], item);
		self.recent[victim_idx] = false;
		hand += 1;
		if hand == len {
			hand = 0;
		}
		self.hand = hand;
		debug!("Bucket.put: policy=GCLOCK evicted_key={:?}", evicted.key);
		(&self.items[victim_idx], Some(evicted))
	}

	/// The index `put` would evict on a full-bucket miss, without mutating.
	///
	/// With the probation ring active: the probation slot at `small_hand` if its
	/// counter is 0, else the main-region sweep victim (min steps, ties by
	/// distance from the main hand). Otherwise a whole-vector scan.
	fn eviction_candidate_index(&self) -> Option<usize> {
		let len = self.items.len();
		if len == 0 {
			return None;
		}
		if self.split_active() {
			let sh = if self.small_hand < self.small_len { self.small_hand } else { 0 };
			if self.items[sh].lfu_counter == 0 {
				return Some(sh);
			}
			return Some(self.scan_min_from(self.small_len, self.main_hand()));
		}
		let start = if self.hand < len { self.hand } else { 0 };
		Some(self.scan_min_from(0, start))
	}

	fn evict_item(&mut self) -> Option<Item<K, V>> {
		let idx = self.eviction_candidate_index()?;
		let evicted = self.remove_at(idx);
		debug!("Bucket.evict_item: policy=S3GCLOCK evicted_key={:?}", evicted.key);
		Some(evicted)
	}

	/// Removes the item at `index`, keeping both hands in bounds.
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		let removed = self.items.remove(index);
		self.recent.remove(index);
		let len = self.items.len();

		if self.hand > index {
			self.hand -= 1;
		}
		if len > self.small_len {
			if self.hand < self.small_len || self.hand >= len {
				self.hand = self.small_len;
			}
		} else {
			self.hand = 0;
		}

		if self.small_hand > index {
			self.small_hand -= 1;
		}
		if self.small_hand >= self.small_len.min(len).max(1) {
			self.small_hand = 0;
		}
		removed
	}
}

impl<K, V> Display for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for item in &self.items {
			writeln!(f, "    {}", item)?;
		}
		Ok(())
	}
}

#[cfg(test)]
mod clock_tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicyKind;

	fn bucket(size: usize) -> AssociativeCacheBucket<i64, i64> {
		AssociativeCacheBucket::new(&AssociativeCacheSettings {
			num_of_buckets: 1,
			bucket_size: size,
			eviction_policy: EvictionPolicyKind::ClassicLRU,
		})
	}

	fn contains(b: &AssociativeCacheBucket<i64, i64>, key: i64) -> bool {
		b.get_items().iter().any(|i| i.key == key)
	}

	#[test]
	fn probation_sizing_table() {
		assert_eq!(probation_len(1), 0);
		assert_eq!(probation_len(3), 0);
		assert_eq!(probation_len(8), 4);
		assert_eq!(probation_len(16), 8);
		assert_eq!(probation_len(64), 32);
		assert_eq!(probation_len(128), 64);
		assert_eq!(probation_len(1024), 256);
		for size in 4..=4096 {
			let s = probation_len(size);
			assert!(s >= 1 && s < size, "size {size}: small_len {s}");
		}
		assert_eq!(bucket(128).small_len, 64);
		assert_eq!(bucket(1024).small_len, 256);
	}

	#[test]
	fn clock_second_chance() {
		let mut b = bucket(3);
		b.put(Item::new(1, 10));
		b.put(Item::new(2, 20));
		b.put(Item::new(3, 30));
		b.get(&1);
		assert_eq!(b.eviction_candidate_index(), Some(1));
		let (ins, ev) = b.put(Item::new(4, 40));
		assert_eq!(ins.key, 4);
		assert_eq!(ev.unwrap().key, 2);
		assert_eq!(b.size(), 3);
		assert!(b.get(&1).is_some());
		assert!(b.get(&2).is_none());
	}

	#[test]
	fn clock_all_referenced_evicts_at_hand() {
		let mut b = bucket(2);
		b.put(Item::new(1, 10));
		b.put(Item::new(2, 20));
		b.get(&1);
		b.get(&2);
		let (_, ev) = b.put(Item::new(3, 30));
		assert_eq!(ev.unwrap().key, 1);
	}

	#[test]
	fn gclock_counter_saturates() {
		let mut b = bucket(2);
		b.put(Item::new(1, 10));
		for _ in 0..10 {
			b.get(&1);
		}
		assert_eq!(b.get(&1).unwrap().lfu_counter, GCLOCK_MAX);
		b.put(Item::new(1, 11));
		assert_eq!(b.get(&1).unwrap().lfu_counter, GCLOCK_MAX);
	}

	#[test]
	fn gclock_counter_saturates_at_seven() {
		let mut b = bucket(2);
		b.put(Item::new(1, 10));
		for _ in 0..20 {
			b.get(&1);
		}
		let pos = b.get_position(&1).unwrap();
		assert_eq!(b.get_items()[pos].lfu_counter, 7);
	}

	#[test]
	fn gclock_frequent_item_outlives_once_hit() {
		let mut b = bucket(2);
		b.put(Item::new(1, 10));
		b.put(Item::new(2, 20));
		b.get(&1);
		b.get(&1);
		b.get(&1); // counter 3, recent -> steps 3
		b.get(&2); // counter 1, recent -> steps 2
		let predicted = b.eviction_candidate_index().unwrap();
		assert_eq!(b.get_items()[predicted].key, 2);
		let (_, ev) = b.put(Item::new(3, 30));
		assert_eq!(ev.unwrap().key, 2);
		assert!(b.get(&1).is_some());
	}

	#[test]
	fn stale_high_count_loses_to_recent_low_count() {
		let mut b = bucket(2);
		assert_eq!(b.small_len, 0);
		b.put(Item::new(1, 10)); // A
		b.put(Item::new(2, 20));
		for _ in 0..7 {
			b.get(&1);
		}
		assert_eq!(b.get_items()[0].lfu_counter, 7);
		// First pass: A's recent bit is cleared, key 2 (never hit) is evicted.
		let (_, ev) = b.put(Item::new(3, 30)); // B
		assert_eq!(ev.unwrap().key, 2);
		let pa = b.get_position(&1).unwrap();
		assert!(!b.recent[pa]);
		assert_eq!(b.get_items()[pa].lfu_counter, 7);
		// B is re-used recently, with a lower count than A.
		for _ in 0..4 {
			b.get(&3);
		}
		let pb = b.get_position(&3).unwrap();
		assert!(b.get_items()[pb].lfu_counter < b.get_items()[pa].lfu_counter);
		assert!(b.steps(pa) < b.steps(pb), "stale A must rank below recent B");
		// Second pass: the stale high-count item is the victim, as predicted.
		let predicted = b.eviction_candidate_index().map(|i| b.get_items()[i].key);
		assert_eq!(predicted, Some(1));
		let (_, ev) = b.put(Item::new(4, 40));
		assert_eq!(ev.map(|e| e.key), predicted);
		assert!(contains(&b, 3));
	}

	#[test]
	fn promotion_clears_recent_bit() {
		let mut b = bucket(8);
		for k in 1..=8 {
			b.put(Item::new(k, k));
		}
		b.put(Item::new(9, 9)); // probation slot 0
		b.get(&9);
		let p9 = b.get_position(&9).unwrap();
		assert!(b.recent[p9]);
		b.put(Item::new(10, 10));
		b.put(Item::new(11, 11));
		b.put(Item::new(12, 12));
		b.put(Item::new(13, 13)); // promotes 9
		let pos9 = b.get_position(&9).unwrap();
		assert!(pos9 >= b.small_len);
		assert_eq!(b.get_items()[pos9].lfu_counter, 0);
		assert!(!b.recent[pos9]);
		// The new item in probation slot 0 has a clear bit too.
		let p13 = b.get_position(&13).unwrap();
		assert!(!b.recent[p13]);
	}

	fn check_candidate_matches_sweep(size: usize, key_space: u64, seed: u64) {
		// Deterministic pseudo-random workload: the predicted candidate must
		// always equal the key `put` evicts.
		let mut b = bucket(size);
		let mut x: u64 = seed;
		for _ in 0..4000 {
			x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
			let key = ((x >> 33) % key_space) as i64;
			if b.get(&key).is_none() {
				let predicted = b.eviction_candidate_index().map(|i| b.get_items()[i].key);
				let full = b.is_full();
				let (ins, ev) = b.put(Item::new(key, key));
				assert_eq!(ins.key, key);
				if full {
					assert_eq!(ev.map(|e| e.key), predicted);
				} else {
					assert!(ev.is_none());
				}
			}
			assert!(b.size() <= size);
			assert_eq!(b.recent.len(), b.size());
			assert!(b.get_items().iter().all(|i| i.lfu_counter <= GCLOCK_MAX));
			assert!(b.hand < b.size().max(1));
			assert!(b.small_hand < b.small_len.min(b.size()).max(1));
		}
	}

	#[test]
	fn candidate_matches_sweep() {
		check_candidate_matches_sweep(5, 12, 12345);
		check_candidate_matches_sweep(16, 40, 987654321);
		check_candidate_matches_sweep(64, 150, 42);
		check_candidate_matches_sweep(128, 300, 7);
		// High-reuse: counters climb to the ceiling.
		check_candidate_matches_sweep(64, 80, 2024);
	}

	#[test]
	fn never_hit_item_evicted_after_small_len_misses() {
		let mut b = bucket(8);
		assert_eq!(b.small_len, 4);
		let small_len = b.small_len as i64;
		for k in 1..=8 {
			b.put(Item::new(k, k));
		}
		// The first small_len misses evict the original probation slots 1..=4.
		for k in 0..small_len {
			let (_, ev) = b.put(Item::new(9 + k, 9 + k));
			assert_eq!(ev.unwrap().key, 1 + k);
		}
		assert!(contains(&b, 9));
		// small_len (4) misses after 9's insertion, 9 is evicted.
		let (_, ev) = b.put(Item::new(9 + small_len, 9 + small_len));
		assert_eq!(ev.unwrap().key, 9);
		assert!(!contains(&b, 9));
	}

	#[test]
	fn probation_hit_item_survives_into_main() {
		let mut b = bucket(8);
		assert_eq!(b.small_len, 4);
		for k in 1..=8 {
			b.put(Item::new(k, k));
		}
		b.put(Item::new(9, 9)); // probation slot 0, evicts 1
		b.get(&9);
		b.put(Item::new(10, 10)); // probation slot 1, evicts 2
		b.put(Item::new(11, 11)); // probation slot 2, evicts 3
		b.put(Item::new(12, 12)); // probation slot 3, evicts 4
		let predicted = b.eviction_candidate_index().map(|i| b.get_items()[i].key);
		let (ins, ev) = b.put(Item::new(13, 13));
		assert_eq!(ins.key, 13);
		let ev = ev.unwrap().key;
		assert_eq!(Some(ev), predicted);
		assert_eq!(ev, 5); // main victim, not 9
		let pos9 = b.get_position(&9).unwrap();
		assert!(pos9 >= b.small_len, "9 should now live in the main region");
		assert_eq!(b.get_items()[pos9].lfu_counter, 0);
		// Further probation churn does not touch 9.
		let (_, ev) = b.put(Item::new(14, 14));
		assert_eq!(ev.unwrap().key, 10);
		let (_, ev) = b.put(Item::new(15, 15));
		assert_eq!(ev.unwrap().key, 11);
		assert!(contains(&b, 9));
	}

	#[test]
	fn evict_item_fixes_hand() {
		let mut b = bucket(2);
		b.put(Item::new(1, 10));
		b.put(Item::new(2, 20));
		b.put(Item::new(3, 30)); // hand -> 1
		assert!(b.evict_item().is_some());
		assert!(b.hand < b.size().max(1));
		assert!(b.evict_item().is_some());
		assert_eq!(b.hand, 0);
		assert!(b.evict_item().is_none());
	}

	#[test]
	fn evict_item_keeps_both_hands_in_bounds() {
		let mut b = bucket(8);
		for k in 1..=12 {
			if b.get(&k).is_none() {
				b.put(Item::new(k, k));
			}
		}
		while b.evict_item().is_some() {
			assert!(b.hand < b.size().max(1));
			assert!(b.small_hand < b.small_len.min(b.size()).max(1));
		}
		assert_eq!(b.size(), 0);
	}
}
