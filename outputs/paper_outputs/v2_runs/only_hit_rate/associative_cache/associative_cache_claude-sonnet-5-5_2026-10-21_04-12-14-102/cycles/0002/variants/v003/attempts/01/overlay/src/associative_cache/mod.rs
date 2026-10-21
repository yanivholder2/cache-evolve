use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::{get_index, hash};
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

	/// The two candidate buckets for a key (power-of-two-choices). The second
	/// index comes from a multiplicative rehash and is forced distinct from the
	/// first when more than one bucket exists.
	fn candidates(&self, key: &K) -> (usize, usize) {
		let n = self.buckets.len();
		let first = get_index(key, n);
		let mixed = (hash(key) as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
		let mut second = ((mixed >> 32) as usize) % n;
		if second == first && n > 1 {
			second = (first + 1) % n;
		}
		(first, second)
	}

	/// Insert a key-value pair into the cache
	pub fn put(&mut self, key: K, value: V) {
		let (b1, b2) = self.candidates(&key);
		// Update in place if already resident in either choice.
		for b in [b1, b2] {
			if self.buckets[b].get_position(&key).is_some() {
				self.buckets[b].put(Item::new(key, value));
				return;
			}
		}
		// Place in the less-loaded bucket; on a tie (including both full) use the first.
		let target = if self.buckets[b2].size() < self.buckets[b1].size() { b2 } else { b1 };
		self.buckets[target].put(Item::new(key, value));
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let (b1, b2) = self.candidates(key);
		let idx = if self.buckets[b1].get_position(key).is_some() {
			b1
		} else if b2 != b1 && self.buckets[b2].get_position(key).is_some() {
			b2
		} else {
			return None;
		};
		Some(&self.buckets[idx].get(key)?.value)
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
