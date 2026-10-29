use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::{debug, trace};

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Bucket container. When the configured policy is `ClassicLRU`, the bucket
/// runs an in-place CLOCK approximation instead of move-to-back: a hit sets
/// the item's reference bit (`lfu_counter = 1`) without reordering, and an
/// insert into a full bucket sweeps a per-bucket hand, clearing set bits until
/// it finds an unreferenced slot, which is replaced in place. All other
/// policies take the original code paths unchanged.
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// CLOCK hand: index of the next slot to inspect on eviction.
	hand: usize,
	/// Cached `eviction_policy.name() == "ClassicLRU"`.
	clock: bool,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let eviction_policy: Box<dyn EvictionPolicy<K, V>> = settings.eviction_policy.build();
		let clock = eviction_policy.name() == "ClassicLRU";
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy,
			hand: 0,
			clock,
		}
	}

	/// CLOCK sweep: clears reference bits starting at `hand` until an
	/// unreferenced slot is found; returns its index and leaves `hand`
	/// pointing at it. Precondition: `items` non-empty. Terminates within two
	/// passes since the first pass clears every bit it skips.
	#[inline]
	fn clock_sweep(&mut self) -> usize {
		let len = self.items.len();
		if self.hand >= len {
			self.hand = 0;
		}
		loop {
			let slot = &mut self.items[self.hand];
			if slot.lfu_counter != 0 {
				slot.lfu_counter = 0;
				self.hand += 1;
				if self.hand == len {
					self.hand = 0;
				}
			} else {
				return self.hand;
			}
		}
	}
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

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if self.clock {
			self.items[position].lfu_counter = 1;
			return &self.items[position];
		}
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if !self.clock {
			// Original default `Bucket::put` behaviour.
			if let Some(position) = self.get_position(&item.key) {
				trace!("Bucket.put: key={:?} update_existing", item.key);
				self.items[position].value = item.value;
				let pushed_item = self.get_from_position(position);
				return (pushed_item, None);
			}
			let evicted_item = if self.is_full() { self.evict_item() } else { None };
			self.items.push(item);
			let pushed_item = self.items.last().expect("bucket invariant: just pushed");
			return (pushed_item, evicted_item);
		}

		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			let slot = &mut self.items[position];
			slot.value = item.value;
			slot.lfu_counter = 1;
			return (&self.items[position], None);
		}

		let mut item = item;
		item.lfu_counter = 0;

		if self.items.len() < self.max_size {
			self.items.push(item);
			let pushed_item = self.items.last().expect("bucket invariant: just pushed");
			return (pushed_item, None);
		}

		if self.items.is_empty() {
			// max_size == 0: nothing can be stored; mirror the default path's
			// behaviour of no eviction (validation forbids this geometry anyway).
			self.items.push(item);
			let pushed_item = self.items.last().expect("bucket invariant: just pushed");
			return (pushed_item, None);
		}

		let victim_idx = self.clock_sweep();
		let old = std::mem::replace(&mut self.items[victim_idx], item);
		self.hand = victim_idx + 1;
		if self.hand == self.items.len() {
			self.hand = 0;
		}
		debug!("Bucket.put: policy=CLOCK evicted_key={:?}", old.key);
		(&self.items[victim_idx], Some(old))
	}

	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if !self.clock {
			let idx = self.eviction_candidate_index()?;
			let evicted = self.remove_at(idx);
			debug!(
				"Bucket.evict_item: policy={} evicted_key={:?}",
				self.eviction_policy.name(),
				evicted.key
			);
			return Some(evicted);
		}
		if self.items.is_empty() {
			return None;
		}
		let idx = self.clock_sweep();
		let evicted = self.remove_at(idx);
		if idx < self.hand {
			self.hand -= 1;
		}
		if self.hand >= self.items.len() {
			self.hand = 0;
		}
		debug!("Bucket.evict_item: policy=CLOCK evicted_key={:?}", evicted.key);
		Some(evicted)
	}

	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		if !self.clock {
			return Some(self.eviction_policy.eviction_candidate_index(&self.items));
		}
		let len = self.items.len();
		let start = if self.hand >= len { 0 } else { self.hand };
		for step in 0..len {
			let idx = (start + step) % len;
			if self.items[idx].lfu_counter == 0 {
				return Some(idx);
			}
		}
		Some(start)
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
