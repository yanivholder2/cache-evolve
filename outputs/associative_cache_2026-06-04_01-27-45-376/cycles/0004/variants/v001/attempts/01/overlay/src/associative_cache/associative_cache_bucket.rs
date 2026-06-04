use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use log::{debug, trace};
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

	/// Gap-fill insertion override.
	///
	/// - Update-in-place branch is unchanged from the trait default.
	/// - On a genuine insertion into a *full* bucket we drop the evicted
	///   item out of its slot and slot the fresh item in at the *same*
	///   index, instead of removing-then-pushing-to-the-back. The items
	///   in positions `idx+1..` therefore keep their relative LRU rank
	///   rather than shifting one step toward the front when the victim
	///   is removed.
	/// - On an insertion into a non-full bucket we keep the trait
	///   default's push-to-back path, so non-full behaviour is identical
	///   to the parent variant (v003).
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			self.get_items_mut()[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		if self.is_full() {
			let idx = self
				.eviction_candidate_index()
				.expect("bucket invariant: is_full implies non-empty");
			let evicted = self.get_items_mut().remove(idx);
			debug!(
				"Bucket.put: gap_fill policy={} evicted_key={:?} at idx={}",
				self.get_eviction_policy().name(),
				evicted.key,
				idx
			);
			self.get_items_mut().insert(idx, item);
			return (&self.get_items()[idx], Some(evicted));
		}

		self.get_items_mut().push(item);
		let pushed_item = self.get_items().last().expect("bucket invariant: just pushed");
		(pushed_item, None)
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
