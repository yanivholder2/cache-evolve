use crate::shared::item::Item;
use crate::shared::traits::bucket::{Bucket, InsertPosition};
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
	/// Per-bucket occupancy counter: the number of occupied slots. Maintained on every
	/// insert (`put_at`), eviction/removal (`remove_at`, `remove`), so fullness checks
	/// read this counter instead of deriving the count from the slot contents.
	/// Raw mutation through `get_items_mut` bypasses it; `is_full` debug-asserts it
	/// still matches the items vector.
	occupied: usize,
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
			occupied: 0,
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

	fn put_at(&mut self, item: Item<K, V>, pos: InsertPosition) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		// Victim search (via `evict_item` -> `remove_at`) only needs the fullness bit,
		// which is answered from the occupancy counter.
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		let pushed_item = match pos {
			InsertPosition::Back => {
				self.items.push(item);
				self.items.last().expect("bucket invariant: just pushed")
			}
			InsertPosition::Front => {
				self.items.insert(0, item);
				self.items.first().expect("bucket invariant: just inserted")
			}
		};
		self.occupied += 1;
		(pushed_item, evicted_item)
	}

	fn remove(&mut self, key: &K) {
		self.items.retain(|item| item.key != *key);
		self.occupied = self.items.len();
	}

	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		self.occupied = self.occupied.saturating_sub(1);
		self.items.remove(index)
	}

	fn is_full(&self) -> bool {
		debug_assert_eq!(
			self.occupied,
			self.items.len(),
			"occupancy counter out of sync with bucket contents"
		);
		self.occupied == self.max_size
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
