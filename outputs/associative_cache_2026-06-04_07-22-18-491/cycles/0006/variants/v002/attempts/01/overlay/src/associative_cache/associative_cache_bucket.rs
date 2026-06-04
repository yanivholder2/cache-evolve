use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Associative cache bucket with a probationary admission zone.
///
/// New inserts enter `probe` (a small FIFO) rather than primary.  An item
/// graduates from probe to primary only when a subsequent `get` confirms
/// reuse.  Items that are never accessed again age out of the probe FIFO
/// without ever occupying a primary slot, leaving primary exclusively for
/// items with confirmed temporal locality.
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// Primary resident items — the only items counted by `size()` / `get_items()`.
	items: Vec<Item<K, V>>,
	/// Maximum number of primary items.
	max_size: usize,
	/// Eviction policy used for primary items.
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Probationary zone: newly inserted items wait here for a confirming `get`.
	probe: Vec<Item<K, V>>,
	/// Maximum number of probe-zone items (≈ one-quarter of primary capacity).
	probe_max_size: usize,
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
			probe: Vec::new(),
			probe_max_size: (settings.bucket_size / 4).max(1),
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

	/// Look up `key`, checking primary first then the probe zone.
	///
	/// A probe hit promotes the item to primary (evicting the LRU primary
	/// resident if primary is full) and gives it an initial `lfu_counter` of 1
	/// so it enters primary with a non-zero frequency head start.
	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		// Primary hit — delegate to the eviction policy (e.g. LRU move-to-back).
		if let Some(position) = self.get_position(key) {
			return Some(self.get_from_position(position));
		}

		// Probe hit — promote to primary on second access.
		if let Some(probe_pos) = self.probe.iter().position(|i| &i.key == key) {
			let mut probe_item = self.probe.remove(probe_pos);
			// Second observed access: give it a non-zero frequency before primary competition.
			probe_item.lfu_counter += 1;

			// Evict from primary if full.
			if self.items.len() == self.max_size {
				let evict_idx = self.eviction_policy.eviction_candidate_index(&self.items);
				self.items.remove(evict_idx);
			}

			self.items.push(probe_item);
			let back = self.items.len() - 1;
			return Some(&self.items[back]);
		}

		None
	}

	/// Insert or update a key in the bucket.
	///
	/// Three cases, in priority order:
	/// 1. Key already in primary → update value in place (no promotion needed).
	/// 2. Key already in probe → update value, stay in probe (promotion is
	///    deferred to `get`).
	/// 3. Brand-new key → enqueue in probe, FIFO-evicting the oldest probe
	///    entry if the probe is full.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Case 1: key already resident in primary — update in place.
		if let Some(position) = self.get_position(&item.key) {
			self.get_items_mut()[position].value = item.value;
			let pushed = self.get_from_position(position);
			return (pushed, None);
		}

		// Case 2: key in probe — update value only; promotion deferred to get().
		if let Some(probe_pos) = self.probe.iter().position(|i| i.key == item.key) {
			self.probe[probe_pos].value = item.value;
			return (&self.probe[probe_pos], None);
		}

		// Case 3: brand-new key — insert into probe, FIFO-evict oldest if full.
		if self.probe.len() >= self.probe_max_size {
			self.probe.remove(0); // oldest probe entry dropped; no stash in this variant
		}
		self.probe.push(item);
		let back = self.probe.len() - 1;
		(&self.probe[back], None)
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
