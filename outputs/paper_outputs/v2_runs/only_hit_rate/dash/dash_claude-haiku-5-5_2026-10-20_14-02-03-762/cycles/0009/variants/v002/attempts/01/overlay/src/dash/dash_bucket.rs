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
	/// Per-bucket occupancy count: equal to the number of occupied slots. Kept in
	/// step with every insert, evict, and removal in this bucket, so victim search
	/// can be gated on it without re-deriving occupancy from the items vector.
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

	/// Same contract as the trait default. The only difference is victim search: it
	/// runs only when the occupancy count equals capacity; below capacity no victim
	/// selection happens and the item is placed directly.
	fn put_at(&mut self, item: Item<K, V>, pos: InsertPosition) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.occupied == self.max_size {
			self.evict_item()
		} else {
			None
		};
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
		let before = self.items.len();
		self.items.retain(|item| item.key != *key);
		self.occupied = self.occupied.saturating_sub(before - self.items.len());
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
