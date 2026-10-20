use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use std::{
	fmt::{Debug, Display, Formatter},
	hash::Hash,
	sync::atomic::{AtomicUsize, Ordering},
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
	/// Rotating cursor that seeds the two-choice victim sample. Advanced once per
	/// eviction decision; atomic so `eviction_candidate_index(&self)` can advance it.
	victim_cursor: AtomicUsize,
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
			victim_cursor: AtomicUsize::new(0),
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

	/// Two-choice frequency-aware victim selection. Two distinct slots are sampled
	/// from a rotating per-bucket cursor and the one with the lower access counter is
	/// evicted; ties go to the lower slot index (the existing bucket order). The cost
	/// per eviction is a constant two comparisons, independent of bucket size.
	fn eviction_candidate_index(&self) -> Option<usize> {
		let n = self.items.len();
		if n == 0 {
			return None;
		}
		if n == 1 {
			return Some(0);
		}
		let c = self.victim_cursor.fetch_add(1, Ordering::Relaxed);
		let a = c % n;
		// Offset in 1..n so that `b` is always distinct from `a`.
		let b = (a + 1 + (c / n) % (n - 1)) % n;
		let pick = match self.items[a].lfu_counter.cmp(&self.items[b].lfu_counter) {
			std::cmp::Ordering::Less => a,
			std::cmp::Ordering::Greater => b,
			std::cmp::Ordering::Equal => a.min(b),
		};
		Some(pick)
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
