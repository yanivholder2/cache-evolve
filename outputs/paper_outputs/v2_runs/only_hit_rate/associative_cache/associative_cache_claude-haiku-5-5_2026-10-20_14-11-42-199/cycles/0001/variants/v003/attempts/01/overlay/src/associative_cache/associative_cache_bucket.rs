use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::{ClassicLRU, EvictionPolicyKind, Fifo, Lfu, Lifo, Swap, TimestampLRU};
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Statically dispatched eviction policy: one variant per in-crate policy.
///
/// Non-generic on purpose: the wrapped policy structs are non-generic and
/// implement `EvictionPolicy<K, V>` for every `K`, `V`, so the enum does too.
#[derive(Debug)]
pub enum BucketPolicy {
	ClassicLRU(ClassicLRU),
	TimestampLRU(TimestampLRU),
	Lifo(Lifo),
	Lfu(Lfu),
	Fifo(Fifo),
	Swap(Swap),
}

impl BucketPolicy {
	fn from_kind(kind: EvictionPolicyKind) -> Self {
		match kind {
			EvictionPolicyKind::ClassicLRU => BucketPolicy::ClassicLRU(ClassicLRU),
			EvictionPolicyKind::TimestampLRU => BucketPolicy::TimestampLRU(TimestampLRU),
			EvictionPolicyKind::Lifo => BucketPolicy::Lifo(Lifo),
			EvictionPolicyKind::Lfu => BucketPolicy::Lfu(Lfu),
			EvictionPolicyKind::Fifo => BucketPolicy::Fifo(Fifo),
			EvictionPolicyKind::Swap => BucketPolicy::Swap(Swap),
		}
	}
}

impl<K, V> EvictionPolicy<K, V> for BucketPolicy
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn name(&self) -> &'static str {
		match self {
			BucketPolicy::ClassicLRU(p) => EvictionPolicy::<K, V>::name(p),
			BucketPolicy::TimestampLRU(p) => EvictionPolicy::<K, V>::name(p),
			BucketPolicy::Lifo(p) => EvictionPolicy::<K, V>::name(p),
			BucketPolicy::Lfu(p) => EvictionPolicy::<K, V>::name(p),
			BucketPolicy::Fifo(p) => EvictionPolicy::<K, V>::name(p),
			BucketPolicy::Swap(p) => EvictionPolicy::<K, V>::name(p),
		}
	}

	fn get_from_position(&self, items: &mut Vec<Item<K, V>>, position: usize) -> usize {
		match self {
			BucketPolicy::ClassicLRU(p) => EvictionPolicy::<K, V>::get_from_position(p, items, position),
			BucketPolicy::TimestampLRU(p) => EvictionPolicy::<K, V>::get_from_position(p, items, position),
			BucketPolicy::Lifo(p) => EvictionPolicy::<K, V>::get_from_position(p, items, position),
			BucketPolicy::Lfu(p) => EvictionPolicy::<K, V>::get_from_position(p, items, position),
			BucketPolicy::Fifo(p) => EvictionPolicy::<K, V>::get_from_position(p, items, position),
			BucketPolicy::Swap(p) => EvictionPolicy::<K, V>::get_from_position(p, items, position),
		}
	}

	fn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		match self {
			BucketPolicy::ClassicLRU(p) => EvictionPolicy::<K, V>::eviction_candidate_index(p, items),
			BucketPolicy::TimestampLRU(p) => EvictionPolicy::<K, V>::eviction_candidate_index(p, items),
			BucketPolicy::Lifo(p) => EvictionPolicy::<K, V>::eviction_candidate_index(p, items),
			BucketPolicy::Lfu(p) => EvictionPolicy::<K, V>::eviction_candidate_index(p, items),
			BucketPolicy::Fifo(p) => EvictionPolicy::<K, V>::eviction_candidate_index(p, items),
			BucketPolicy::Swap(p) => EvictionPolicy::<K, V>::eviction_candidate_index(p, items),
		}
	}

	fn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		match self {
			BucketPolicy::ClassicLRU(p) => EvictionPolicy::<K, V>::promotion_candidate_index(p, items),
			BucketPolicy::TimestampLRU(p) => EvictionPolicy::<K, V>::promotion_candidate_index(p, items),
			BucketPolicy::Lifo(p) => EvictionPolicy::<K, V>::promotion_candidate_index(p, items),
			BucketPolicy::Lfu(p) => EvictionPolicy::<K, V>::promotion_candidate_index(p, items),
			BucketPolicy::Fifo(p) => EvictionPolicy::<K, V>::promotion_candidate_index(p, items),
			BucketPolicy::Swap(p) => EvictionPolicy::<K, V>::promotion_candidate_index(p, items),
		}
	}
}

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: BucketPolicy,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy: BucketPolicy::from_kind(settings.eviction_policy),
		}
	}
}

impl<K, V> Bucket<K, V> for AssociativeCacheBucket<K, V>
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
		&self.eviction_policy
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = EvictionPolicy::<K, V>::get_from_position(&self.eviction_policy, &mut self.items, position);
		&self.items[index]
	}
}

impl<K, V> Display for AssociativeCacheBucket<K, V>
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
