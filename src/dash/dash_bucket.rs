// TODO: finish module documentation
//! Bucket implementation for the Dash cache.
//!
//!
//! # Examples
//!
//! ```
//! ```

use crate::settings::EvictionPolicy;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
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
	// TODO: consider using a linked list for O(1) changes
	items: Vec<Item<K, V>>,
	max_size: usize,
	// TODO: make this a reference with a lifetime
	eviction_policy: EvictionPolicy,
}

impl<K, V> DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(max_size: usize, eviction_policy: EvictionPolicy) -> Self {
		DashBucket {
			// TODO: consider creating a vector with a fixed size for better performance after initialization
			items: Vec::new(),
			max_size,
			eviction_policy,
		}
	}
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

	fn get_eviction_policy(&self) -> &EvictionPolicy {
		&self.eviction_policy
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

// TODO: implement
/*
#[cfg(test)]
mod tests {
}
*/
