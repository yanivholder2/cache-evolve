use crate::shared::item::Item;
use crate::shared::traits::bucket::{lru_get_from_position, Bucket};
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
	/// Cached once at construction: whether `eviction_policy` is ClassicLRU. Lets
	/// the hot get/put path branch on a plain `bool` field and call the inlined
	/// free functions instead of paying a vtable indirection per access.
	is_classic_lru: bool,
}

impl<K, V> DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(max_size: usize, eviction_policy: Box<dyn EvictionPolicy<K, V>>) -> Self {
		let is_classic_lru = eviction_policy.name() == "ClassicLRU";
		DashBucket {
			items: Vec::with_capacity(max_size),
			max_size,
			eviction_policy,
			is_classic_lru,
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

	fn is_classic_lru(&self) -> bool {
		self.is_classic_lru
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		// Devirtualized hot path: for ClassicLRU, call the inlined remove+push
		// directly instead of dispatching through the trait object. Every other
		// policy keeps the existing virtual call. Behaviour is identical.
		let index = if self.is_classic_lru {
			lru_get_from_position(&mut self.items, position)
		} else {
			self.eviction_policy.get_from_position(&mut self.items, position)
		};
		&self.items[index]
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
