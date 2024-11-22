// use crate::shared::item::Item;
// use crate::shared::settings::EvictionPolicy;
// use crate::shared::traits::bucket::Bucket;
// use crate::shared::traits::cache::Cache;
// use crate::shared::traits::segment::Segment;
// use associative_cache_settings::AssociativeCacheSettings;
// use std::hash::Hash;

// pub mod associative_cache_jni;
// mod associative_cache_settings;

// #[derive(Debug)]
// pub struct AssociativeCache<K, V>
// where
// 	K: Hash + Eq + Copy,
// 	V: Eq + Copy,
// {
// 	pub segments: Vec<AssociativeCacheSegment<K, V>>,
// }

// impl<K, V> AssociativeCache<K, V>
// where
// 	K: Hash + Eq + Copy,
// 	V: Eq + Copy,
// {
// 	pub fn new(settings: AssociativeCacheSettings) -> Self {
// 		let mut segments = Vec::new();
// 		for _ in 0..settings.size {
// 			segments.push(AssociativeCacheSegment::new(settings.clone()));
// 		}
// 		Self { segments }
// 	}
// }

// impl<K, V> Cache<K, V> for AssociativeCache<K, V>
// where
// 	K: Hash + Eq + Copy,
// 	V: Eq + Copy,
// {
// 	type S = AssociativeCacheSegment<K, V>;

// 	fn get_segments_mut(&mut self) -> &mut Vec<Self::S> {
// 		&mut self.segments
// 	}
// }

// #[derive(Debug)]
// pub struct AssociativeCacheSegment<K, V>
// where
// 	K: Hash + Eq + Copy,
// 	V: Eq + Copy,
// {
// 	pub buckets: Vec<AssociativeCacheBucket<K, V>>,
// 	pub segment_size: usize,
// }

// impl<K, V> AssociativeCacheSegment<K, V>
// where
// 	K: Hash + Eq + Copy,
// 	V: Eq + Copy,
// {
// 	pub fn new(settings: AssociativeCacheSettings) -> Self {
// 		let mut buckets = Vec::new();
// 		for _ in 0..settings.segment_size {
// 			buckets.push(AssociativeCacheBucket::new(settings.clone()));
// 		}
// 		Self {
// 			buckets,
// 			segment_size: settings.segment_size,
// 		}
// 	}
// }

// impl<K, V> Segment<K, V> for AssociativeCacheSegment<K, V>
// where
// 	K: Hash + Eq + Copy,
// 	V: Eq + Copy,
// {
// 	type B = AssociativeCacheBucket<K, V>;

// 	fn get_buckets(&self) -> &Vec<Self::B> {
// 		&self.buckets
// 	}

// 	fn get_buckets_mut(&mut self) -> &mut Vec<Self::B> {
// 		&mut self.buckets
// 	}

// 	fn get_segment_size(&self) -> usize {
// 		self.segment_size
// 	}
// }

// #[derive(Debug)]
// pub struct AssociativeCacheBucket<K, V>
// where
// 	K: Hash + Eq + Copy,
// 	V: Eq + Copy,
// {
// 	items: Vec<Item<K, V>>,
// 	max_size: usize,
// 	eviction_policy: EvictionPolicy,
// }

// impl<K, V> AssociativeCacheBucket<K, V>
// where
// 	K: Hash + Eq + Copy,
// 	V: Eq + Copy,
// {
// 	pub fn new(settings: AssociativeCacheSettings) -> Self {
// 		Self {
// 			items: Vec::new(),
// 			max_size: settings.bucket_size,
// 			eviction_policy: settings.eviction_policy,
// 		}
// 	}
// }

// impl<K, V> Bucket<K, V> for AssociativeCacheBucket<K, V>
// where
// 	K: Hash + Eq + Copy,
// 	V: Eq + Copy,
// {
// 	fn get_items(&self) -> &Vec<Item<K, V>> {
// 		&self.items
// 	}

// 	fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>> {
// 		&mut self.items
// 	}

// 	fn get_max_size(&self) -> usize {
// 		self.max_size
// 	}

// 	fn get_eviction_policy(&self) -> &EvictionPolicy {
// 		&self.eviction_policy
// 	}

// 	fn get_and_update_lru_item(&mut self, position: usize) -> &Item<K, V> {
// 		let item = self.items.remove(position);
// 		self.items.push(item);
// 		self.items.last().unwrap()
// 	}

// 	fn evict_lru_item(&mut self) -> Option<Item<K, V>> {
// 		// TODO: this is in O(n). there could be a more performant way to do that
// 		Some(self.items.remove(0))
// 	}
// }
