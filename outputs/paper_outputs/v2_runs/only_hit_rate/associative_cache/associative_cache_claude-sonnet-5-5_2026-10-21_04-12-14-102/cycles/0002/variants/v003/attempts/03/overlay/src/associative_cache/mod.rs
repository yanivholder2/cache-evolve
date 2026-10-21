use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::hash;
use associative_cache_bucket::AssociativeCacheBucket;
use associative_cache_settings::AssociativeCacheSettings;
use log::info;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

pub mod associative_cache_bucket;
pub mod associative_cache_jni;
pub mod associative_cache_settings;

#[derive(Debug)]
pub struct AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub buckets: Vec<AssociativeCacheBucket<K, V>>,
}

impl<K, V> AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn try_new(settings: AssociativeCacheSettings) -> Option<Self> {
		if let Err(problems) = settings.validate() {
			log::error!("AssociativeCache.try_new: validation failed: {}", problems.join(", "));
			return None;
		}
		info!("AssociativeCache.try_new: settings={settings:?}");
		let mut buckets = Vec::with_capacity(settings.num_of_buckets);
		for _ in 0..settings.num_of_buckets {
			buckets.push(AssociativeCacheBucket::new(&settings));
		}
		Some(Self { buckets })
	}

	pub fn new(settings: AssociativeCacheSettings) -> Self {
		Self::try_new(settings).expect("AssociativeCache::new: initialization failed")
	}

	/// Two candidate bucket indices from two different mixes of the key hash.
	fn two_indices(&self, key: &K) -> (usize, usize) {
		let n = self.buckets.len();
		let h = hash(key) as u64;
		let i1 = (h % n as u64) as usize;
		let mixed = h.wrapping_mul(0x9E37_79B9_7F4A_7C15);
		let mixed = (mixed ^ (mixed >> 32)).wrapping_mul(0xD6E8_FEB8_6659_FD93);
		let i2 = ((mixed >> 17) % n as u64) as usize;
		(i1, i2)
	}

	/// Insert a key-value pair into the cache (power-of-two-choices placement).
	pub fn put(&mut self, key: K, value: V) {
		let (i1, i2) = self.two_indices(&key);
		// Defensive: update in place if already resident in either bucket.
		if self.buckets[i1].get_position(&key).is_some() {
			self.buckets[i1].put(Item::new(key, value));
			return;
		}
		if i2 != i1 && self.buckets[i2].get_position(&key).is_some() {
			self.buckets[i2].put(Item::new(key, value));
			return;
		}
		let target = if i1 == i2 {
			i1
		} else {
			let s1 = self.buckets[i1].size();
			let s2 = self.buckets[i2].size();
			if s1 < s2 {
				i1
			} else if s2 < s1 {
				i2
			} else if self.buckets[i1].is_full() {
				// both full: evict from the bucket with the older front entry
				let t1 = self.buckets[i1].get_items().first().map(|i| i.timestamp);
				let t2 = self.buckets[i2].get_items().first().map(|i| i.timestamp);
				match (t1, t2) {
					(Some(a), Some(b)) if b < a => i2,
					_ => i1,
				}
			} else {
				i1
			}
		};
		self.buckets[target].put(Item::new(key, value));
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let (i1, i2) = self.two_indices(key);
		let idx = if self.buckets[i1].get_position(key).is_some() {
			i1
		} else if i2 != i1 && self.buckets[i2].get_position(key).is_some() {
			i2
		} else {
			return None;
		};
		let data = self.buckets[idx].get(key)?;
		Some(&data.value)
	}

	/// Remove a key from whichever bucket holds it.
	pub fn remove(&mut self, key: &K) -> Option<Item<K, V>> {
		let (i1, i2) = self.two_indices(key);
		for idx in [i1, i2] {
			if let Some(pos) = self.buckets[idx].get_position(key) {
				return Some(self.buckets[idx].remove_at(pos));
			}
		}
		None
	}
}

impl<K, V> framework::Cache<K, V> for AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn put(&mut self, key: K, value: V) {
		AssociativeCache::put(self, key, value);
	}

	fn get(&mut self, key: &K) -> Option<&V> {
		AssociativeCache::get_and_update_item(self, key)
	}
}

impl<K, V> Display for AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for bucket in &self.buckets {
			writeln!(f, "{}", bucket)?;
		}
		Ok(())
	}
}
