use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// Per-slot one-hit-wonder protection bit, kept index-aligned with `items`
	/// (`reuse.len() == items.len()` at every method exit). A freshly admitted
	/// miss item is unprotected (`false`); any access via `get_from_position`
	/// promotes its slot to `true`. This lives on the bucket struct, which the
	/// metadata-size metric does not weigh, so `Item` stays key+value = 16 bytes.
	reuse: Vec<bool>,
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
			reuse: Vec::with_capacity(settings.bucket_size),
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

	/// Relocates the accessed item per the policy, then mirrors the move onto the
	/// `reuse` bit with the *same shape* of operation — remove the bit from
	/// `position` and re-insert it at the index the policy returned — and marks
	/// the slot protected. This is O(n) (a single remove + insert on a tiny
	/// bucket), not the O(n^2) key search of a rebuild.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		let bit = self.reuse.remove(position);
		self.reuse.insert(index, bit);
		self.reuse[index] = true;
		&self.items[index]
	}

	/// Scan-resistance by compounding two signals (H02): a cache-miss item is
	/// admitted at the **front (index 0)** on probation *and* with its `reuse`
	/// bit `false`. A never-reused scan item is therefore both positionally
	/// exposed and reuse-unprotected, so it is the first evicted; an item hit
	/// before the next conflicting insert is promoted (moved per policy and
	/// `reuse = true`), shielding it. Update-in-place of an existing key routes
	/// through `get_from_position`, keeping the per-policy access behaviour and
	/// marking the slot protected.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		// Front admission: insert the miss item at the probationary index 0,
		// unprotected. Eviction (above) already removed the policy/reuse
		// candidate first, so capacity is conserved and alignment preserved.
		self.items.insert(0, item);
		self.reuse.insert(0, false);
		let pushed_item = self.items.first().expect("bucket invariant: just inserted");
		(pushed_item, evicted_item)
	}

	/// Prefer evicting the frontmost probationary one-hit-wonder: the first slot
	/// in index order whose `reuse` bit is `false`. If every resident has been
	/// protected, fall back to index 0.
	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		Some(self.reuse.iter().position(|&protected| !protected).unwrap_or(0))
	}

	/// Remove the item and its aligned `reuse` bit together.
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		self.reuse.remove(index);
		self.items.remove(index)
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
