use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use std::{
	fmt::{Debug, Display, Formatter},
	hash::Hash,
};

/// Saturation cap for the per-item access counter (`Item::lfu_counter`) that the
/// segment bumps on each target-bucket hit. Victim selection prefers the lowest
/// counter, so the cap keeps the signal coarse: a few hits separate an item from a
/// one-shot arrival without letting old popularity pin it forever.
pub const HIT_COUNTER_CAP: usize = 3;

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

	/// Frequency-aware victim selection: evict the resident with the lowest access
	/// counter. Ties go to the policy's own candidate (the parent's existing order),
	/// so with no counter signal this reduces to the policy's eviction choice.
	///
	/// The counter is unsigned, so 0 is the global minimum. Once a zero-counter slot
	/// exists, the victim is the policy's candidate if that candidate is also at 0,
	/// otherwise the first zero slot in slot order. That is exactly what the full
	/// min-scan returns, so the scan stops at the first zero instead of visiting
	/// every slot. The full scan only runs when no resident is at 0.
	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		let policy_idx = self.eviction_policy.eviction_candidate_index(&self.items);
		if self.items[policy_idx].lfu_counter == 0 {
			return Some(policy_idx);
		}
		if let Some(zero_idx) = self.items.iter().position(|it| it.lfu_counter == 0) {
			return Some(zero_idx);
		}
		let min_counter = self.items.iter().map(|it| it.lfu_counter).min()?;
		if self.items[policy_idx].lfu_counter == min_counter {
			return Some(policy_idx);
		}
		self.items.iter().position(|it| it.lfu_counter == min_counter)
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
