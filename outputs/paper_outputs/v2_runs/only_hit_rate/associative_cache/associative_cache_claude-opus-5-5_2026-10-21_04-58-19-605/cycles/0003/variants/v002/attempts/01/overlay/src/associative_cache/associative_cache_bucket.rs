use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use log::debug;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Insertion depth for new (missed) keys under ClassicLRU, as a fraction of the
/// bucket length measured from the eviction end (index 0). 1/4 = a quarter of
/// the way up from the eviction end (between midpoint insertion and pure LIP).
const INSERT_DEPTH_NUM: usize = 1;
const INSERT_DEPTH_DEN: usize = 4;

/// Number of slots nearest the eviction end (index 0) considered as eviction
/// victims under ClassicLRU. The lowest hit-count item in this window is evicted.
const VICTIM_WINDOW: usize = 4;
/// Saturation cap for the per-item hit counter.
const HIT_COUNT_MAX: usize = 15;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// When true, new keys are inserted at INSERT_DEPTH_NUM/INSERT_DEPTH_DEN of
	/// the bucket instead of the MRU tail (LIP/BIP-style), hits are counted, and
	/// eviction picks the least-hit item among the VICTIM_WINDOW slots nearest
	/// the eviction end. Only enabled for ClassicLRU.
	midpoint_insert: bool,
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
			midpoint_insert: settings.eviction_policy == EvictionPolicyKind::ClassicLRU,
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
		if self.midpoint_insert {
			let c = &mut self.items[position].lfu_counter;
			if *c < HIT_COUNT_MAX {
				*c += 1;
			}
		}
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if !self.midpoint_insert || self.items.is_empty() {
			// Trait-default path.
			let idx = self.eviction_candidate_index()?;
			let evicted = self.remove_at(idx);
			debug!(
				"Bucket.evict_item: policy={} evicted_key={:?}",
				self.get_eviction_policy().name(),
				evicted.key
			);
			return Some(evicted);
		}

		let window = VICTIM_WINDOW.min(self.items.len());
		// Lowest counter wins; strict `<` keeps ties at the lowest index (recency tie-break).
		let mut victim = 0;
		for i in 1..window {
			if self.items[i].lfu_counter < self.items[victim].lfu_counter {
				victim = i;
			}
		}

		// Aging: a zero-count victim while other window items have hits means
		// those hits are protecting stale items; halve them.
		if self.items[victim].lfu_counter == 0
			&& (0..window).any(|i| i != victim && self.items[i].lfu_counter > 0)
		{
			for i in 0..window {
				if i != victim {
					self.items[i].lfu_counter >>= 1;
				}
			}
		}

		let evicted = self.remove_at(victim);
		debug!(
			"Bucket.evict_item: policy={} (freq-window) evicted_key={:?}",
			self.get_eviction_policy().name(),
			evicted.key
		);
		Some(evicted)
	}

	fn put(&mut self, mut item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Existing key: baseline behaviour (update value, promote per policy).
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		// Capacity check runs before the insert, so size never exceeds max_size.
		let evicted_item = if self.is_full() { self.evict_item() } else { None };

		if !self.midpoint_insert {
			// Baseline miss path: push to the tail.
			self.items.push(item);
			let pushed_item = self.items.last().expect("bucket invariant: just pushed");
			return (pushed_item, evicted_item);
		}

		// Every newcomer starts with zero hits.
		item.lfu_counter = 0;

		// Depth insertion: computed after the eviction. With a full 16-slot
		// bucket, len = 15 here, so idx = 3.
		let idx = self.items.len() * INSERT_DEPTH_NUM / INSERT_DEPTH_DEN;
		self.items.insert(idx, item);
		(&self.items[idx], evicted_item)
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
