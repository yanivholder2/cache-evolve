use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use log::debug;
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

	/// Eviction with ClassicLRU on-demand aging.
	///
	/// For every policy other than ClassicLRU this is identical to the trait
	/// default: ask the policy for a victim index and remove it.
	///
	/// For ClassicLRU we override the candidate selection so we can mutate
	/// `items` when the bucket is fully touched (which the trait method
	/// cannot, since it only has `&[Item]`). The flow is:
	///
	/// 1. Scan left-to-right for the first slot with `lfu_counter == 0` —
	///    v003's "untouched-first" rule. If found, evict it.
	/// 2. Otherwise the bucket is in the all-touched regime. Halve every
	///    slot's counter in place (`>>= 1`). The slot(s) with the previously
	///    smallest non-zero counter (necessarily 1, since the minimum is at
	///    least 1) drop to 0, so at least one zero-counter slot now exists.
	///    Slots whose counter was 2 land at 1, etc., preserving the relative
	///    rank order between long-lived re-referenced items.
	/// 3. Re-scan left-to-right for `lfu_counter == 0` and evict it.
	/// 4. As a safety belt — should be unreachable after the aging pass —
	///    evict slot 0, matching v003's final fallback.
	///
	/// Aging happens only here at the eviction regime boundary, never on the
	/// `get_from_position` hit path, so genuine re-references keep accumulating
	/// counter as in v003.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}

		if self.eviction_policy.name() != "ClassicLRU" {
			// Preserve the trait-default behaviour for all other policies.
			let idx = self.eviction_policy.eviction_candidate_index(&self.items);
			let evicted = self.items.remove(idx);
			debug!(
				"Bucket.evict_item: policy={} evicted_key={:?}",
				self.eviction_policy.name(),
				evicted.key
			);
			return Some(evicted);
		}

		// ClassicLRU: prefer leftmost slot that was never accessed.
		if let Some(i) = self.items.iter().position(|it| it.lfu_counter == 0) {
			let evicted = self.items.remove(i);
			debug!(
				"Bucket.evict_item: policy=ClassicLRU evicted_key={:?}",
				evicted.key
			);
			return Some(evicted);
		}

		// All-touched regime: halve every counter in place. After this pass
		// at least one slot has `lfu_counter == 0` (the previously-lowest one,
		// whose value was 1 → 0; higher values shift down by one bit and
		// retain rank order).
		for item in self.items.iter_mut() {
			item.lfu_counter >>= 1;
		}

		// Re-scan left-to-right after aging.
		if let Some(i) = self.items.iter().position(|it| it.lfu_counter == 0) {
			let evicted = self.items.remove(i);
			debug!(
				"Bucket.evict_item: policy=ClassicLRU aged evicted_key={:?}",
				evicted.key
			);
			return Some(evicted);
		}

		// Unreachable in practice (the aging pass guarantees a zero), but mirror
		// v003's final fallback of evicting slot 0.
		let evicted = self.items.remove(0);
		debug!(
			"Bucket.evict_item: policy=ClassicLRU fallback evicted_key={:?}",
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
