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

	/// Hot-promotion (H01): every new item lands at the MRU (`Back`) end of the
	/// bucket, ignoring the requested `InsertPosition`.
	///
	/// In the running cache the only caller that ever asks for `Front` is the
	/// promotion path threaded through `promotion_get` (stash→target promote,
	/// target→probing promote, and the demotion re-inserts); the new-key `put`
	/// path already inserts at `Back`. Forcing `Back` here therefore turns the
	/// cold-promotion segment into a hot-promotion one: a promoted item lands at
	/// the protected MRU end (the ClassicLRU `promotion_candidate_index`, i.e.
	/// `len - 1`) instead of straight onto the eviction-candidate end
	/// (`eviction_candidate_index == 0`, the `Front`). A just-promoted hot key is
	/// thus protected from immediate eviction, and one further access promotes it
	/// to the next tier. Eviction-when-full and single-eviction-per-put are
	/// preserved, so invariants P1–P5 continue to hold.
	fn put_at(&mut self, item: Item<K, V>, _pos: InsertPosition) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		self.items.push(item);
		let pushed_item = self.items.last().expect("bucket invariant: just pushed");
		(pushed_item, evicted_item)
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
