use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use std::{
	fmt::{Debug, Display, Formatter},
	hash::Hash,
};

/// "Seen again" flag, packed into the top bit of the existing `lfu_counter`
/// metadata word (unused by ClassicLRU) so Item size stays unchanged.
const SEEN_FLAG: usize = 1usize << (usize::BITS - 1);

#[derive(Debug)]
pub struct DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Second-chance flagging only applies to ClassicLRU (the only policy that
	/// leaves `lfu_counter` untouched).
	use_flag: bool,
}

impl<K, V> DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(max_size: usize, eviction_policy: Box<dyn EvictionPolicy<K, V>>) -> Self {
		let use_flag = eviction_policy.name() == "ClassicLRU";
		DashBucket {
			items: Vec::with_capacity(max_size),
			max_size,
			eviction_policy,
			use_flag,
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
		if self.use_flag {
			self.items[index].lfu_counter |= SEEN_FLAG;
		}
		&self.items[index]
	}

	/// One-bit second chance: evict the first (LRU-most) item whose seen flag is
	/// clear; if every item is flagged, clear all flags and evict the first.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}
		let idx = if self.use_flag {
			match self.items.iter().position(|it| it.lfu_counter & SEEN_FLAG == 0) {
				Some(i) => i,
				None => {
					for it in self.items.iter_mut() {
						it.lfu_counter &= !SEEN_FLAG;
					}
					0
				}
			}
		} else {
			self.eviction_candidate_index()?
		};
		Some(self.remove_at(idx))
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
