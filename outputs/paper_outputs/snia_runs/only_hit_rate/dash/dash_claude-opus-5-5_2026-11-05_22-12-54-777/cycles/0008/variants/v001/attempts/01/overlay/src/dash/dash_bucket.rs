use crate::shared::eviction_policy::consume_ghost_exemptions;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::debug;
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
	/// H01: the policy is the frequency tail scan (`ClassicLRU`), whose victim
	/// choice honours the one-time ghost-admission exemption.
	tail_scan: bool,
}

impl<K, V> DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(max_size: usize, eviction_policy: Box<dyn EvictionPolicy<K, V>>) -> Self {
		let tail_scan = eviction_policy.name() == "ClassicLRU";
		DashBucket {
			items: Vec::with_capacity(max_size),
			max_size,
			eviction_policy,
			tail_scan,
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

	/// H01: the policy's victim already skips a ghost-admitted min candidate;
	/// here every flagged item that was skipped consumes its exemption before
	/// the victim is removed.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}
		let idx = self.eviction_policy.eviction_candidate_index(&self.items);
		if self.tail_scan {
			consume_ghost_exemptions(&mut self.items, idx);
		}
		let evicted = self.items.remove(idx);
		debug!(
			"Bucket.evict_item: policy={} evicted_key={:?}",
			self.eviction_policy.name(),
			evicted.key
		);
		Some(evicted)
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicyKind;

	#[test]
	fn ghost_admitted_tail_item_survives_one_eviction() {
		let mut b = DashBucket::<i64, i64>::new(3, EvictionPolicyKind::ClassicLRU.build());
		let mut g = Item::new(1, 1);
		g.lfu_counter = 1;
		g.set_ghost_admitted();
		b.put(g);
		let mut x = Item::new(2, 2);
		x.lfu_counter = 2;
		b.put(x);
		let mut y = Item::new(3, 3);
		y.lfu_counter = 3;
		b.put(y);
		let (_, ev) = b.put(Item::new(4, 4));
		assert_eq!(ev.unwrap().key, 2, "the flagged min candidate is exempted once");
		let pos = b.get_position(&1).unwrap();
		assert!(!b.get_items()[pos].is_ghost_admitted(), "exemption consumed");
		// Key 1 (counter 1) is now the min with no protection left.
		b.get_items_mut().retain(|it| it.key != 4);
		let mut z = Item::new(7, 7);
		z.lfu_counter = 5;
		b.put(z);
		let (_, ev) = b.put(Item::new(8, 8));
		assert_eq!(ev.unwrap().key, 1, "a second eviction choice is not exempted");
	}
}
