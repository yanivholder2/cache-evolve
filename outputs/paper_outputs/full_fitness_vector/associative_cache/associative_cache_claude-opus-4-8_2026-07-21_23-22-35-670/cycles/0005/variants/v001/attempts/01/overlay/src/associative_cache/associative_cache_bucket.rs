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

	/// Single-step promotion (H01): a hit at `position` climbs exactly **one**
	/// slot toward the back (`items.swap(position, position + 1)`), rather than
	/// jumping straight to the MRU back as `ClassicLRU::get_from_position` would.
	/// Combined with front admission (index 0) and front eviction (index 0),
	/// this makes a hot key *earn* its way out of the eviction zone: a single
	/// touch of a one-hit scan item only nudges it one slot and it is still
	/// evicted soon, while a genuinely reused key ratchets toward the protected
	/// back over repeated hits. This overrides the per-policy promotion geometry
	/// with a positional single-swap — O(1), no per-`Item` state. The item at the
	/// back stays put (no `position + 1` slot), so the geometry saturates cleanly.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = if position + 1 < self.items.len() {
			self.items.swap(position, position + 1);
			position + 1
		} else {
			position
		};
		&self.items[index]
	}

	/// Positional scan-resistance (H03): a cache-miss item is admitted at the
	/// **front (index 0)** — the immediate eviction position — placing it on
	/// probation rather than in the protected back region. Under `ClassicLRU`
	/// (eval policy), repeated hits promote the item toward the back via
	/// `get_from_position` (single-step, per H01); a never-reused scan item stays
	/// at the front and is the first evicted, shielding previously-hot items
	/// clustered near the back. Update-in-place of an existing key routes through
	/// the same `get_from_position`, so an update also single-steps toward the back.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		// Front admission: insert at the probationary index 0 rather than pushing
		// to the back (MRU). Eviction still removes the policy's candidate first,
		// so capacity is conserved.
		self.items.insert(0, item);
		let pushed_item = self.items.first().expect("bucket invariant: just inserted");
		(pushed_item, evicted_item)
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
