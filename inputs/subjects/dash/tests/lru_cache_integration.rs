use dash::lru_cache::LruCache;
use rand::Rng;

// ==================== Constants ====================

const SMALL_CACHE_CAPACITY: usize = 16;
const LARGE_CACHE_CAPACITY: usize = 256;
const STRESS_NUM_OPS: usize = 10_000;
const STRESS_KEY_RANGE: i64 = 100;
const VALUES_CHECK_CAPACITY: usize = 64;
const VALUES_CHECK_NUM_OPS: usize = 5_000;
const VALUES_CHECK_KEY_RANGE: i64 = 200;
const FILL_TEST_NUM_ITEMS: i64 = 100;

// ==================== Helpers ====================

fn tiny_cache() -> LruCache<i64, i64> {
	LruCache::new(2)
}

fn small_cache() -> LruCache<i64, i64> {
	LruCache::new(SMALL_CACHE_CAPACITY)
}

// ==================== Basic CRUD ====================

#[test]
fn get_miss_returns_none() {
	let mut cache = tiny_cache();
	assert!(cache.get(&42).is_none());
}

#[test]
fn put_then_get() {
	let mut cache = tiny_cache();
	cache.put(1, 10);
	assert_eq!(*cache.get(&1).unwrap(), 10);
}

#[test]
fn put_overwrite() {
	let mut cache = tiny_cache();
	cache.put(1, 10);
	cache.put(1, 20);
	assert_eq!(*cache.get(&1).unwrap(), 20);
}

#[test]
fn multiple_keys() {
	let mut cache = small_cache();
	for i in 0..SMALL_CACHE_CAPACITY as i64 {
		cache.put(i, i * 10);
	}
	for i in 0..SMALL_CACHE_CAPACITY as i64 {
		assert_eq!(*cache.get(&i).unwrap(), i * 10);
	}
}

// ==================== Eviction ====================

#[test]
fn eviction_removes_lru_item() {
	let mut cache = tiny_cache();
	cache.put(1, 10);
	cache.put(2, 20);
	cache.put(3, 30); // evicts 1
	assert!(cache.get(&1).is_none());
	assert_eq!(*cache.get(&2).unwrap(), 20);
	assert_eq!(*cache.get(&3).unwrap(), 30);
}

#[test]
fn get_promotes_to_mru() {
	let mut cache = tiny_cache();
	cache.put(1, 10);
	cache.put(2, 20);
	cache.get(&1); // promotes 1 to MRU
	cache.put(3, 30); // evicts 2 (LRU)
	assert_eq!(*cache.get(&1).unwrap(), 10);
	assert!(cache.get(&2).is_none());
	assert_eq!(*cache.get(&3).unwrap(), 30);
}

#[test]
fn put_existing_promotes_to_mru() {
	let mut cache = tiny_cache();
	cache.put(1, 10);
	cache.put(2, 20);
	cache.put(1, 11); // update promotes 1 to MRU
	cache.put(3, 30); // evicts 2 (LRU)
	assert_eq!(*cache.get(&1).unwrap(), 11);
	assert!(cache.get(&2).is_none());
	assert_eq!(*cache.get(&3).unwrap(), 30);
}

#[test]
fn eviction_chain() {
	let mut cache = LruCache::new(3);
	cache.put(1, 10);
	cache.put(2, 20);
	cache.put(3, 30);
	// Each new insert evicts the oldest
	cache.put(4, 40); // evicts 1
	cache.put(5, 50); // evicts 2
	cache.put(6, 60); // evicts 3
	assert!(cache.get(&1).is_none());
	assert!(cache.get(&2).is_none());
	assert!(cache.get(&3).is_none());
	assert_eq!(*cache.get(&4).unwrap(), 40);
	assert_eq!(*cache.get(&5).unwrap(), 50);
	assert_eq!(*cache.get(&6).unwrap(), 60);
}

// ==================== Settings / capacity ====================

#[test]
fn capacity_one() {
	let mut cache = LruCache::new(1);
	cache.put(1, 10);
	assert_eq!(*cache.get(&1).unwrap(), 10);
	cache.put(2, 20);
	assert!(cache.get(&1).is_none());
	assert_eq!(*cache.get(&2).unwrap(), 20);
}

#[test]
fn large_capacity() {
	let mut cache: LruCache<i64, i64> = LruCache::new(LARGE_CACHE_CAPACITY);
	for i in 0..LARGE_CACHE_CAPACITY as i64 {
		cache.put(i, i * 10);
	}
	// All items should still be present
	for i in 0..LARGE_CACHE_CAPACITY as i64 {
		assert_eq!(*cache.get(&i).unwrap(), i * 10);
	}
}

// ==================== Fill beyond capacity ====================

#[test]
fn fill_beyond_capacity_then_query() {
	let mut cache: LruCache<i64, i64> = LruCache::new(SMALL_CACHE_CAPACITY);
	for i in 0..FILL_TEST_NUM_ITEMS {
		cache.put(i, i * 10);
	}
	// Only the last `capacity` items should survive (strict LRU, no hash routing)
	for i in 0..FILL_TEST_NUM_ITEMS {
		if i >= FILL_TEST_NUM_ITEMS - SMALL_CACHE_CAPACITY as i64 {
			assert_eq!(*cache.get(&i).unwrap(), i * 10, "recent key {} should be present", i);
		} else {
			assert!(cache.get(&i).is_none(), "old key {} should have been evicted", i);
		}
	}
}

// ==================== Stress ====================

#[test]
fn random_workload_no_panic() {
	let mut rng = rand::thread_rng();
	let mut cache = small_cache();
	for _ in 0..STRESS_NUM_OPS {
		let key: i64 = rng.gen_range(0..STRESS_KEY_RANGE);
		if rng.gen_bool(0.5 /* probability */) {
			cache.put(key, key * 10);
		} else {
			cache.get(&key);
		}
	}
	// Verify cache is still functional
	let sentinel = STRESS_KEY_RANGE + 1;
	cache.put(sentinel, sentinel * 10);
	assert_eq!(*cache.get(&sentinel).unwrap(), sentinel * 10);
}

#[test]
fn random_workload_values_correct() {
	let mut rng = rand::thread_rng();
	let mut cache: LruCache<i64, i64> = LruCache::new(VALUES_CHECK_CAPACITY);
	for _ in 0..VALUES_CHECK_NUM_OPS {
		let key: i64 = rng.gen_range(0..VALUES_CHECK_KEY_RANGE);
		if rng.gen_bool(0.5 /* probability */) {
			cache.put(key, key * 10);
		} else if let Some(&val) = cache.get(&key) {
			assert_eq!(val, key * 10, "value mismatch for key {}", key);
		}
	}
}
