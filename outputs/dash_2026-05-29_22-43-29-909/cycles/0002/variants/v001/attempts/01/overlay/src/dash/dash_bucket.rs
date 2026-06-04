use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use std::{
	fmt::{Debug, Display, Formatter},
	hash::Hash,
};

#[derive(Debug)]
pub struct DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
}

impl<K, V> DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(max_size: usize, eviction_policy: Box<dyn EvictionPolicy<K, V>>) -> Self {
		DashBucket {
			items: Vec::with_capacity(max_size),
			max_size,
			eviction_policy,
		}
	}
}

pub fn new_bucket<K, V>(
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
) -> Box<dyn Bucket<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	Box::new(DashBucket::new(max_size, eviction_policy))
}

impl<K, V> Bucket<K, V> for DashBucket<K, V>
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

	/// Frequency retention (H01): a `remove_at` driven by `promotion_get` is a *hit*
	/// on the resident (a stash-hit or a hottest-target-hit being promoted up a tier),
	/// so accumulate its access counter before it carries over into the next tier. This
	/// mirrors the per-hit increment that `get_from_position` applies on the
	/// touch-in-place branches, giving the frequency-based promotion trigger a complete
	/// per-item access count across all hit branches (stash, target, probing).
	/// Eviction removals do not flow through here (see `evict_item`), so a demoted
	/// victim is never granted a spurious access.
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		self.items[index].lfu_counter += 1;
		self.items.remove(index)
	}

	/// Evicts the policy's victim *without* counting it as an access. Eviction is not a
	/// hit, so it must not bump `lfu_counter` — otherwise items evicted from a normal
	/// bucket and demoted down the tier cascade would carry an inflated frequency,
	/// muddying the promotion signal. Mirrors the trait default's victim selection and
	/// return value, only skipping the per-hit increment that the overridden
	/// `remove_at` now applies.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		let idx = self.eviction_candidate_index()?;
		Some(self.items.remove(idx))
	}
}

impl<K, V> Display for DashBucket<K, V>
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
