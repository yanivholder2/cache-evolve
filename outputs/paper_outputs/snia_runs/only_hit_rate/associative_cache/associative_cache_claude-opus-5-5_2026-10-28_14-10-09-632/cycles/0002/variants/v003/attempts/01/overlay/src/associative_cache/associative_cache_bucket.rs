use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::{debug, trace};

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Saturation ceiling for the GCLOCK reference counter (2 bits of state).
const GCLOCK_MAX: usize = 3;

/// Bucket using an in-place GCLOCK (generalised CLOCK) scheme.
///
/// `Item::lfu_counter` is reused as a saturating reference counter in
/// `0..=GCLOCK_MAX`, so items keep their size. A hit increments the counter
/// (saturating at `GCLOCK_MAX`); items never move. On a full-bucket miss the
/// hand sweeps, decrementing non-zero counters, until it finds an item whose
/// counter is 0, then replaces that item in place. New items start at 0.
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// CLOCK hand. Invariant: `hand < max(items.len(), 1)`.
	hand: usize,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
			hand: 0,
		}
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

	/// GCLOCK hit: saturating counter increment, no reordering, no policy dispatch.
	#[inline]
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let item = &mut self.items[position];
		item.lfu_counter = gclock_bump(item.lfu_counter);
		&self.items[position]
	}

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Existing key: overwrite value, bump counter, no movement.
		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			let slot = &mut self.items[position];
			slot.value = item.value;
			slot.lfu_counter = gclock_bump(slot.lfu_counter);
			return (&self.items[position], None);
		}

		let len = self.items.len();
		if len < self.max_size {
			let mut item = item;
			item.lfu_counter = 0;
			self.items.push(item);
			let pushed = self.items.last().expect("bucket invariant: just pushed");
			return (pushed, None);
		}

		// Full (len == max_size). max_size == 0 is rejected by settings
		// validation, so len > 0 here.
		debug_assert!(len > 0, "GCLOCK put on zero-capacity bucket");
		let mut hand = if self.hand < len { self.hand } else { 0 };
		// Sweep: each full rotation decrements every non-zero counter, so this
		// terminates within GCLOCK_MAX rotations (plus one partial).
		while self.items[hand].lfu_counter > 0 {
			self.items[hand].lfu_counter -= 1;
			hand += 1;
			if hand == len {
				hand = 0;
			}
		}
		let victim_idx = hand;
		let mut item = item;
		item.lfu_counter = 0;
		let evicted = std::mem::replace(&mut self.items[victim_idx], item);
		hand += 1;
		if hand == len {
			hand = 0;
		}
		self.hand = hand;
		debug!("Bucket.put: policy=GCLOCK evicted_key={:?}", evicted.key);
		(&self.items[victim_idx], Some(evicted))
	}

	/// The index the GCLOCK sweep would pick, without mutating anything: the
	/// item with the minimum counter, ties broken by distance from `hand`
	/// (wrapping). An item at distance `d` with counter `c` is reached at 0
	/// after `c * len + d` sweep steps, so this is exactly the sweep's victim.
	fn eviction_candidate_index(&self) -> Option<usize> {
		let len = self.items.len();
		if len == 0 {
			return None;
		}
		let start = if self.hand < len { self.hand } else { 0 };
		let mut best = start;
		let mut best_counter = self.items[start].lfu_counter;
		if best_counter == 0 {
			return Some(best);
		}
		for step in 1..len {
			let idx = (start + step) % len;
			let c = self.items[idx].lfu_counter;
			if c < best_counter {
				best = idx;
				best_counter = c;
				if c == 0 {
					break;
				}
			}
		}
		Some(best)
	}

	fn evict_item(&mut self) -> Option<Item<K, V>> {
		let idx = self.eviction_candidate_index()?;
		let evicted = self.remove_at(idx);
		debug!("Bucket.evict_item: policy=GCLOCK evicted_key={:?}", evicted.key);
		Some(evicted)
	}

	/// Removes the item at `index`, keeping the hand pointing at the same
	/// logical successor and within `max(len, 1)`.
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		let removed = self.items.remove(index);
		if self.hand > index {
			self.hand -= 1;
		}
		if self.hand >= self.items.len() {
			self.hand = 0;
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
	fn gclock_frequent_item_outlives_once_hit() {
		let mut b = bucket(2);
		b.put(Item::new(1, 10));
		b.put(Item::new(2, 20));
		b.get(&1);
		b.get(&1);
		b.get(&1); // counter 3
		b.get(&2); // counter 1
		let predicted = b.eviction_candidate_index().unwrap();
		assert_eq!(b.get_items()[predicted].key, 2);
		let (_, ev) = b.put(Item::new(3, 30));
		assert_eq!(ev.unwrap().key, 2);
		assert!(b.get(&1).is_some());
	}

	#[test]
	fn candidate_matches_sweep() {
		// Deterministic pseudo-random workload: the predicted candidate must
		// always equal the key the sweep evicts.
		let mut b = bucket(5);
		let mut x: u64 = 12345;
		for _ in 0..2000 {
			x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
			let key = ((x >> 33) % 12) as i64;
			if b.get(&key).is_none() {
				let predicted = b.eviction_candidate_index().map(|i| b.get_items()[i].key);
				let full = b.is_full();
				let (_, ev) = b.put(Item::new(key, key));
				if full {
					assert_eq!(ev.map(|e| e.key), predicted);
				} else {
					assert!(ev.is_none());
				}
			}
			assert!(b.get_items().iter().all(|i| i.lfu_counter <= GCLOCK_MAX));
			assert!(b.hand < b.size().max(1));
		}
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
}
