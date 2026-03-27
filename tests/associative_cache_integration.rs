mod common;

use common::find_keys_for_index;
use dash::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use dash::associative_cache::AssociativeCache;
use dash::eviction_policy::EvictionPolicy;
use rand::Rng;
use std::thread;
use std::time::Duration;

// ==================== Helpers ====================

/// Single bucket, size 2. All keys map to the same bucket — no hash routing.
fn tiny_cache(policy: EvictionPolicy) -> AssociativeCache<i64, i64> {
	AssociativeCache::new(AssociativeCacheSettings {
		num_of_buckets: 1,
		bucket_size: 2,
		eviction_policy: policy,
	})
}

fn small_cache_settings(policy: EvictionPolicy) -> AssociativeCacheSettings {
	AssociativeCacheSettings {
		num_of_buckets: 4,
		bucket_size: 4,
		eviction_policy: policy,
	}
}

// ==================== Basic CRUD x all policies ====================

#[test]
fn get_miss_returns_none_all_policies() {
	for policy in EvictionPolicy::all() {
		let mut cache = tiny_cache(policy);
		assert!(
			cache.get_and_update_item(&42).is_none(),
			"expected miss on empty cache for {:?}",
			policy
		);
	}
}

#[test]
fn put_then_get_all_policies() {
	for policy in EvictionPolicy::all() {
		let mut cache = tiny_cache(policy);
		cache.put(1, 10);
		assert_eq!(
			*cache
				.get_and_update_item(&1)
				.unwrap_or_else(|| panic!("get returned None for {:?}", policy)),
			10
		);
	}
}

#[test]
fn put_overwrite_all_policies() {
	for policy in EvictionPolicy::all() {
		let mut cache = tiny_cache(policy);
		cache.put(1, 10);
		cache.put(1, 20);
		assert_eq!(
			*cache
				.get_and_update_item(&1)
				.unwrap_or_else(|| panic!("get returned None for {:?}", policy)),
			20
		);
	}
}

// ==================== Eviction per policy ====================
// tiny_cache: 1 bucket, size 2. Overflow on 3rd insert.
// No stash/migration — eviction is immediate and observable.

#[test]
fn eviction_fifo() {
	// FIFO: evicts front (oldest). Insert order: 1,2,3 → evicts 1.
	let mut cache = tiny_cache(EvictionPolicy::Fifo);
	cache.put(1, 10);
	cache.put(2, 20);
	cache.put(3, 30); // evicts 1
	assert!(cache.get_and_update_item(&1).is_none());
	assert_eq!(*cache.get_and_update_item(&2).unwrap(), 20);
	assert_eq!(*cache.get_and_update_item(&3).unwrap(), 30);
}

#[test]
fn eviction_lifo() {
	// LIFO: evicts back (newest). Insert 1,2,3 → evicts 2.
	let mut cache = tiny_cache(EvictionPolicy::Lifo);
	cache.put(1, 10);
	cache.put(2, 20);
	cache.put(3, 30); // pops back=2
	assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
	assert!(cache.get_and_update_item(&2).is_none());
	assert_eq!(*cache.get_and_update_item(&3).unwrap(), 30);
}

#[test]
fn eviction_classic_lru() {
	// Insert 1,2. Touch 1 (moves to back = MRU). Insert 3 → evicts front=2 (LRU).
	let mut cache = tiny_cache(EvictionPolicy::ClassicLRU);
	cache.put(1, 10);
	cache.put(2, 20); // bucket=[1,2]
	cache.get_and_update_item(&1); // ClassicLRU: 1 moves to back → [2,1]
	cache.put(3, 30); // evicts front=2
	assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
	assert!(cache.get_and_update_item(&2).is_none());
	assert_eq!(*cache.get_and_update_item(&3).unwrap(), 30);
}

#[test]
fn eviction_timestamp_lru() {
	// Insert 1,2. Touch 1 (refreshes timestamp). Insert 3 → evicts oldest timestamp = 2.
	let mut cache = tiny_cache(EvictionPolicy::TimestampLRU);
	cache.put(1, 10);
	thread::sleep(Duration::from_millis(20));
	cache.put(2, 20);
	thread::sleep(Duration::from_millis(20));
	cache.get_and_update_item(&1); // refresh 1's timestamp
	cache.put(3, 30); // evicts 2 (oldest timestamp)
	assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
	assert!(cache.get_and_update_item(&2).is_none());
	assert_eq!(*cache.get_and_update_item(&3).unwrap(), 30);
}

#[test]
fn eviction_lfu() {
	// Insert 1,2. Access 1 twice (counter=2). Insert 3 → evicts 2 (counter=0, lowest).
	let mut cache = tiny_cache(EvictionPolicy::Lfu);
	cache.put(1, 10);
	cache.put(2, 20);
	cache.get_and_update_item(&1); // counter: 1
	cache.get_and_update_item(&1); // counter: 2
	cache.put(3, 30); // evicts min counter = 2 (counter=0)
	assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
	assert!(cache.get_and_update_item(&2).is_none());
	assert_eq!(*cache.get_and_update_item(&3).unwrap(), 30);
}

#[test]
fn eviction_swap() {
	// Insert 1,2. get(2) swaps to front → [2,1]. Insert 3 → evicts front=2.
	let mut cache = tiny_cache(EvictionPolicy::Swap);
	cache.put(1, 10);
	cache.put(2, 20); // bucket=[1,2]
	cache.get_and_update_item(&2); // Swap: 2 swaps with 1 → [2,1]
	cache.put(3, 30); // evicts front=2
	assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
	assert!(cache.get_and_update_item(&2).is_none());
	assert_eq!(*cache.get_and_update_item(&3).unwrap(), 30);
}

// ==================== Settings ====================

#[test]
fn default_settings_works() {
	let mut cache: AssociativeCache<i64, i64> = AssociativeCache::new(AssociativeCacheSettings::default());
	cache.put(1, 10);
	assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
}

#[test]
fn custom_settings_large_buckets() {
	let settings = AssociativeCacheSettings {
		num_of_buckets: 8,
		bucket_size: 32,
		eviction_policy: EvictionPolicy::ClassicLRU,
	};
	let capacity = settings.num_of_buckets * settings.bucket_size;
	let num_items = 200;
	let mut cache: AssociativeCache<i64, i64> = AssociativeCache::new(settings);
	for i in 0..num_items {
		cache.put(i, i * 10);
	}
	let found: usize = (0..num_items)
		.filter(|i| cache.get_and_update_item(i).is_some())
		.count();
	// Hash distribution may cause a few evictions in overloaded buckets
	let min_expected = capacity * 3 / 4;
	assert!(
		found >= min_expected,
		"expected >= {} items retrievable, got {}",
		min_expected,
		found
	);
}

// ==================== Multi-bucket routing ====================

#[test]
fn keys_route_to_different_buckets() {
	let settings = small_cache_settings(EvictionPolicy::Fifo);
	let num_buckets = settings.num_of_buckets;
	let mut cache: AssociativeCache<i64, i64> = AssociativeCache::new(settings);

	let bucket_keys: Vec<Vec<i64>> = (0..num_buckets)
		.map(|b| find_keys_for_index(b, num_buckets, 1 /* count */))
		.collect();
	assert!(
		bucket_keys.iter().all(|v| !v.is_empty()),
		"could not find a key for every bucket; distribution: {:?}",
		bucket_keys.iter().map(|v| v.len()).collect::<Vec<_>>()
	);

	for &k in bucket_keys.iter().flatten() {
		cache.put(k, k * 10);
	}
	for &k in bucket_keys.iter().flatten() {
		assert_eq!(
			*cache.get_and_update_item(&k).expect("key should be present after put"),
			k * 10
		);
	}
}

// ==================== Stress ====================

#[test]
fn random_workload_no_panic() {
	let num_ops = 10_000;
	let key_range = 100;
	let mut rng = rand::thread_rng();
	for policy in EvictionPolicy::all() {
		let mut cache: AssociativeCache<i64, i64> = AssociativeCache::new(small_cache_settings(policy));
		for _ in 0..num_ops {
			let key: i64 = rng.gen_range(0..key_range);
			if rng.gen_bool(0.5 /* probability */) {
				cache.put(key, key * 10);
			} else {
				cache.get_and_update_item(&key);
			}
		}
		// Verify cache is still functional after random workload
		let sentinel_key = key_range + 1;
		let sentinel_val = sentinel_key * 10;
		cache.put(sentinel_key, sentinel_val);
		assert_eq!(
			*cache
				.get_and_update_item(&sentinel_key)
				.unwrap_or_else(|| panic!("cache broken after random workload for {:?}", policy)),
			sentinel_val
		);
	}
}

#[test]
fn fill_beyond_capacity_then_query() {
	let settings = small_cache_settings(EvictionPolicy::ClassicLRU);
	let capacity = settings.num_of_buckets * settings.bucket_size;
	let num_items: i64 = 1000;
	let mut cache: AssociativeCache<i64, i64> = AssociativeCache::new(settings);
	for i in 0..num_items {
		cache.put(i, i * 10);
	}
	let mut found = 0;
	for i in 0..num_items {
		if let Some(&val) = cache.get_and_update_item(&i) {
			assert_eq!(val, i * 10, "value mismatch for key {}", i);
			found += 1;
		}
	}
	// Recent items should survive under LRU
	let min_expected = capacity / 2;
	assert!(
		found >= min_expected,
		"expected >= {} hits from {}-slot cache, got {}",
		min_expected,
		capacity,
		found
	);
}
