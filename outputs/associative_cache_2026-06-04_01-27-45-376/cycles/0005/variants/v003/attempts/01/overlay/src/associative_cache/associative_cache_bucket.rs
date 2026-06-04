use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::debug;

use std::cell::Cell;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Running count of evictions performed by this bucket. Used by the
	/// `evict_item` override below to fire a periodic newest-slot sacrifice
	/// (every `max_size` evictions) under `ClassicLRU`, providing
	/// scan-resistance on top of v003's two-pass filter without touching
	/// per-item state. `Cell` keeps the field metadata-neutral with respect
	/// to potential `&self` callers (matching the vehicle validated in an
	/// earlier cohort).
	evict_count: Cell<u32>,
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
			evict_count: Cell::new(0),
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
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	/// Periodic newest-slot sacrifice for scan-resistance under `ClassicLRU`.
	///
	/// For policies other than `ClassicLRU`, this replicates the trait
	/// default verbatim (pick `eviction_candidate_index`, remove it). Under
	/// `ClassicLRU`, the bucket increments `evict_count` and, every
	/// `max_size` evictions, bypasses v003's filter to drop the newest
	/// (back) slot directly — pruning scan-pattern keys at their freshest
	/// before they crowd out long-resident items at the front. All other
	/// evictions continue to use v003's two-pass `lfu_counter == 0` filter
	/// via the trait default path.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		// Non-ClassicLRU policies: behave exactly like the trait default.
		if self.eviction_policy.name() != "ClassicLRU" {
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

		// ClassicLRU: alternate between v003's filter and a newest-slot
		// sacrifice on a fixed cadence K = bucket capacity.
		let n = self.evict_count.get().wrapping_add(1);
		self.evict_count.set(n);
		let k = self.max_size.max(1) as u32;

		let evicted = if n % k == 0 && self.items.len() >= 2 {
			let last = self.items.len() - 1;
			self.items.remove(last)
		} else {
			let idx = self
				.eviction_policy
				.eviction_candidate_index(&self.items);
			self.items.remove(idx)
		};
		debug!(
			"Bucket.evict_item: policy={} evicted_key={:?}",
			self.eviction_policy.name(),
			evicted.key
		);
		Some(evicted)
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
