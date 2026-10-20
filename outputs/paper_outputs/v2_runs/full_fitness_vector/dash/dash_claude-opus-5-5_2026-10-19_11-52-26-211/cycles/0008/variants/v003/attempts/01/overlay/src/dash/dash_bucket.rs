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

/// Boxed constructor, kept for any caller that wants a `Box<dyn Bucket>`.
#[allow(dead_code)]
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

/// By-value constructor: lets `SegmentCore` store buckets contiguously in a
/// `Vec<DashBucket<K, V>>` with statically dispatched method calls.
#[inline]
pub fn new_bucket_concrete<K, V>(
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
) -> DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	DashBucket::new(max_size, eviction_policy)
}

// ── Inherent, statically dispatched hot-path methods ─────────────────────────
// Inherent methods take precedence over the trait's, so `SegmentCore` call sites
// (`buckets[i].get_position(..)` etc.) resolve here with no vtable and no import.
// Each mirrors the `Bucket` trait's default semantics exactly.
#[allow(dead_code)]
impl<K, V> DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	#[inline]
	pub fn get_items(&self) -> &Vec<Item<K, V>> {
		&self.items
	}

	#[inline]
	pub fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>> {
		&mut self.items
	}

	#[inline]
	pub fn get_max_size(&self) -> usize {
		self.max_size
	}

	#[inline]
	pub fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	/// Plain key-equality scan over the items slice.
	#[inline]
	pub fn get_position(&self, key: &K) -> Option<usize> {
		self.items.iter().position(|d| d.key == *key)
	}

	#[inline]
	pub fn size(&self) -> usize {
		self.items.len()
	}

	#[inline]
	pub fn is_full(&self) -> bool {
		self.items.len() == self.max_size
	}

	/// Touches the item per the eviction policy (recency update) and returns it.
	#[inline]
	pub fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	#[inline]
	pub fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		self.items.remove(index)
	}

	#[inline]
	pub fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		Some(self.eviction_policy.eviction_candidate_index(&self.items))
	}

	#[inline]
	pub fn promotion_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		Some(self.eviction_policy.promotion_candidate_index(&self.items))
	}

	/// Delegates to the trait's default `put_at` (statically dispatched on `Self`).
	#[inline]
	pub fn put_at(&mut self, item: Item<K, V>, pos: InsertPosition) -> (&Item<K, V>, Option<Item<K, V>>) {
		<Self as Bucket<K, V>>::put_at(self, item, pos)
	}

	#[inline]
	pub fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		<Self as Bucket<K, V>>::put_at(self, item, InsertPosition::Back)
	}

	#[inline]
	pub fn evict_item(&mut self) -> Option<Item<K, V>> {
		<Self as Bucket<K, V>>::evict_item(self)
	}

	#[inline]
	pub fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let position = self.get_position(key)?;
		Some(self.get_from_position(position))
	}

	#[inline]
	pub fn remove(&mut self, key: &K) {
		self.items.retain(|item| item.key != *key);
	}
}

impl<K, V> Bucket<K, V> for DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	#[inline]
	fn get_items(&self) -> &Vec<Item<K, V>> {
		&self.items
	}

	#[inline]
	fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>> {
		&mut self.items
	}

	#[inline]
	fn get_max_size(&self) -> usize {
		self.max_size
	}

	#[inline]
	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	#[inline]
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	#[inline]
	fn get_position(&self, key: &K) -> Option<usize> {
		self.items.iter().position(|d| d.key == *key)
	}

	#[inline]
	fn is_full(&self) -> bool {
		self.items.len() == self.max_size
	}

	#[inline]
	fn size(&self) -> usize {
		self.items.len()
	}

	#[inline]
	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		Some(self.eviction_policy.eviction_candidate_index(&self.items))
	}

	#[inline]
	fn promotion_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		Some(self.eviction_policy.promotion_candidate_index(&self.items))
	}

	#[inline]
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
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
