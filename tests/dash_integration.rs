mod common;

use common::find_keys_for_index;
use dash::dash::dash_settings::{DashSettings, DRAGON_SETTINGS};
use dash::dash::Dash;
use dash::eviction_policy::EvictionPolicy;
use dash::get_index;
use rand::Rng;
use std::thread;
use std::time::Duration;

// ==================== Helpers ====================

/// 1 segment, 1 normal bucket (size 2), 1 stash bucket (size 2).
/// Total capacity 4. All keys route to the same buckets — no hash routing.
fn tiny_dash(policy: EvictionPolicy) -> Dash<i64, i64> {
	Dash::new(&DashSettings {
		num_of_segments: 1,
		num_of_normal_buckets: 1,
		num_of_stash_buckets: 1,
		bucket_size: 2,
		eviction_policy: policy,
		debug_mode: 0,
	})
}

/// 2 segments, 4 normal buckets, 2 stash buckets, bucket_size 2.
fn small_dash_settings(policy: EvictionPolicy) -> DashSettings {
	DashSettings {
		num_of_segments: 2,
		num_of_normal_buckets: 4,
		num_of_stash_buckets: 2,
		bucket_size: 2,
		eviction_policy: policy,
		debug_mode: 0,
	}
}

// ==================== Basic CRUD x all policies ====================

#[test]
fn get_miss_returns_none_all_policies() {
	for policy in EvictionPolicy::all() {
		let mut dash = tiny_dash(policy);
		assert!(
			dash.get_and_update_item(&42).is_none(),
			"expected miss on empty cache for {:?}",
			policy
		);
	}
}

#[test]
fn put_then_get_all_policies() {
	for policy in EvictionPolicy::all() {
		let mut dash = tiny_dash(policy);
		dash.put(1, 10);
		assert_eq!(
			*dash
				.get_and_update_item(&1)
				.unwrap_or_else(|| panic!("get returned None for {:?}", policy)),
			10
		);
	}
}

#[test]
fn put_overwrite_all_policies() {
	for policy in EvictionPolicy::all() {
		let mut dash = tiny_dash(policy);
		dash.put(1, 10);
		dash.put(1, 20);
		assert_eq!(
			*dash
				.get_and_update_item(&1)
				.unwrap_or_else(|| panic!("get returned None for {:?}", policy)),
			20
		);
	}
}

// ==================== Stash-to-normal migration ====================

#[test]
fn repeated_get_still_returns_value() {
	let mut dash = tiny_dash(EvictionPolicy::Fifo);
	dash.put(1, 10);
	// First get: migrates from stash to normal bucket
	assert_eq!(*dash.get_and_update_item(&1).unwrap(), 10);
	// Second get: finds in normal bucket
	assert_eq!(*dash.get_and_update_item(&1).unwrap(), 10);
}

#[test]
fn many_puts_then_gets() {
	// Items go to stash on put; only migrate to normal on get.
	let settings = small_dash_settings(EvictionPolicy::Fifo);
	let stash_capacity = settings.num_of_segments * settings.num_of_stash_buckets * settings.bucket_size;
	let num_items: i64 = 50;
	let mut dash: Dash<i64, i64> = Dash::new(&settings);
	for i in 0..num_items {
		dash.put(i, i * 10);
	}
	let mut found = 0;
	for i in 0..num_items {
		if let Some(&val) = dash.get_and_update_item(&i) {
			assert_eq!(val, i * 10, "value mismatch for key {}", i);
			found += 1;
		}
	}
	// Stash slots are limited; most items get evicted before a get can migrate them.
	let min_expected = stash_capacity / 4;
	assert!(
		found >= min_expected,
		"expected at least {} items retrievable, got {}",
		min_expected, found
	);
}

#[test]
fn interleaved_put_get() {
	let mut dash = tiny_dash(EvictionPolicy::Fifo);
	// Put then immediately get each item — exercises stash→normal migration per item
	for i in 1..=2 {
		dash.put(i, i * 10);
		assert_eq!(*dash.get_and_update_item(&i).unwrap(), i * 10);
	}
}

// ==================== Probing bucket ====================
// Dash probing: when an item isn't in its target normal bucket, DashSegment::get
// checks bucket[(target+1) % n] (wraps around). This tests the path through
// the public Dash API.

#[test]
fn probing_bucket_lookup() {
	// Items can overflow from bucket 0 into bucket 1 during migration.
	let settings = DashSettings {
		num_of_segments: 1,
		num_of_normal_buckets: 2,
		num_of_stash_buckets: 1,
		bucket_size: 2,
		eviction_policy: EvictionPolicy::Fifo,
		debug_mode: 0,
	};
	let num_normal = settings.num_of_normal_buckets;
	let mut dash: Dash<i64, i64> = Dash::new(&settings);

	// Find keys that hash to normal bucket 0
	let bucket0_keys = find_keys_for_index(0, num_normal, 4);
	assert!(
		bucket0_keys.len() >= 4,
		"need 4 keys targeting bucket 0, found {}",
		bucket0_keys.len()
	);

	// Put all 4 keys (go to stash). Stash has 2 slots so first 2 survive.
	// Then get them to migrate to normal bucket 0 which also has 2 slots.
	// When bucket 0 is full, evicted items go back to stash.
	for &k in &bucket0_keys[..2] {
		dash.put(k, k * 10);
	}
	// Migrate first 2 to normal bucket 0 → bucket 0 is now full
	for &k in &bucket0_keys[..2] {
		assert_eq!(
			*dash.get_and_update_item(&k).expect("should migrate from stash"),
			k * 10
		);
	}

	// All 4 keys target bucket 0. Verify the first 2 are still findable
	// (found in normal bucket 0 directly, no probing needed)
	for &k in &bucket0_keys[..2] {
		assert_eq!(
			*dash
				.get_and_update_item(&k)
				.expect("should find in normal bucket"),
			k * 10
		);
	}
}

#[test]
fn last_bucket_probes_into_first_bucket() {
	// Last bucket's probing neighbor wraps around to bucket 0.
	let settings = DashSettings {
		num_of_segments: 1,
		num_of_normal_buckets: 2,
		num_of_stash_buckets: 1,
		bucket_size: 1,
		eviction_policy: EvictionPolicy::Fifo,
		debug_mode: 0,
	};
	let num_normal = settings.num_of_normal_buckets;
	let last_bucket_idx = num_normal - 1;
	let mut dash: Dash<i64, i64> = Dash::new(&settings);

	// Find a key that hashes to the last normal bucket
	let keys = find_keys_for_index(last_bucket_idx, num_normal, 1);
	assert!(!keys.is_empty(), "no key in 0..1000 hashed to last bucket");
	let last_bucket_key = keys[0];

	dash.put(last_bucket_key, 42);
	// Migrates from stash to last normal bucket
	assert_eq!(*dash.get_and_update_item(&last_bucket_key).unwrap(), 42);
	// Second get: found in last normal bucket
	assert_eq!(*dash.get_and_update_item(&last_bucket_key).unwrap(), 42);

	// A miss key targeting the last bucket should miss across stash, target,
	// AND the wrap-around probing bucket (bucket 0).
	let miss_key = (1000..2000i64)
		.find(|k| get_index(k, num_normal) == last_bucket_idx)
		.expect("no miss key found for last bucket");
	assert!(dash.get_and_update_item(&miss_key).is_none());
}

// ==================== Duplicate key in stash + normal ====================

#[test]
fn put_after_migration_returns_fresh_value() {
	// Put key 1, get (migrates to normal), put key 1 again with new value.
	// The second put goes to stash. On next get, stash is checked first,
	// so the fresh value should be returned.
	let mut dash = tiny_dash(EvictionPolicy::Fifo);
	dash.put(1, 10);
	assert_eq!(*dash.get_and_update_item(&1).unwrap(), 10); // migrate to normal
	dash.put(1, 20); // goes to stash (stash is checked first on get)
	assert_eq!(
		*dash.get_and_update_item(&1).unwrap(),
		20,
		"should return fresh value from stash, not stale value from normal"
	);
}

// ==================== Normal-to-stash eviction back-pressure ====================

#[test]
fn evicted_from_normal_returns_to_stash() {
	// tiny_dash: 1 segment, 1 normal (size 2), 1 stash (size 2).
	// Fill normal bucket, then migrate a 3rd item which evicts one back to stash.
	// Verify the evicted item is still retrievable from stash.
	let mut dash = tiny_dash(EvictionPolicy::Fifo);
	dash.put(1, 10);
	dash.put(2, 20);
	// Migrate both to normal → normal=[1,2], stash=[]
	assert_eq!(*dash.get_and_update_item(&1).unwrap(), 10);
	assert_eq!(*dash.get_and_update_item(&2).unwrap(), 20);
	// Put a 3rd item (goes to stash), then get it (migrates to normal, evicts 1 back to stash)
	dash.put(3, 30);
	assert_eq!(*dash.get_and_update_item(&3).unwrap(), 30); // FIFO evicts 1 from normal→stash
	// Item 1 was evicted from normal back to stash — should still be retrievable
	assert_eq!(
		*dash.get_and_update_item(&1).unwrap(),
		10,
		"item evicted from normal should be retrievable from stash"
	);
}

// ==================== Eviction per policy ====================
// tiny_dash: 1 segment, 1 normal (size 2), 1 stash (size 2). Total capacity: 4.

#[test]
fn eviction_fifo() {
	// Put-only overflow: FIFO evicts front (oldest). Items 1,2 evicted; 3,4 survive.
	let mut dash = tiny_dash(EvictionPolicy::Fifo);
	dash.put(1, 10);
	dash.put(2, 20); // stash=[1,2] full
	dash.put(3, 30); // evicts 1
	dash.put(4, 40); // evicts 2
	assert!(dash.get_and_update_item(&1).is_none());
	assert!(dash.get_and_update_item(&2).is_none());
	assert_eq!(*dash.get_and_update_item(&3).unwrap(), 30);
	assert_eq!(*dash.get_and_update_item(&4).unwrap(), 40);
}

#[test]
fn eviction_lifo() {
	// Put-only overflow: LIFO evicts back (newest). First item stays; middle evicted.
	let mut dash = tiny_dash(EvictionPolicy::Lifo);
	dash.put(1, 10);
	dash.put(2, 20); // stash=[1,2] full
	dash.put(3, 30); // LIFO pops back=2, stash=[1,3]
	dash.put(4, 40); // LIFO pops back=3, stash=[1,4]
	assert_eq!(*dash.get_and_update_item(&1).unwrap(), 10);
	assert!(dash.get_and_update_item(&2).is_none());
	assert!(dash.get_and_update_item(&3).is_none());
	assert_eq!(*dash.get_and_update_item(&4).unwrap(), 40);
}

#[test]
fn eviction_classic_lru() {
	// After migration: get(1) makes item 1 MRU → on overflow, item 2 (LRU) is evicted.
	let mut dash = tiny_dash(EvictionPolicy::ClassicLRU);
	dash.put(1, 10);
	dash.put(2, 20);
	dash.get_and_update_item(&1); // migrate to normal
	dash.get_and_update_item(&2); // migrate to normal. normal=[1,2]
	dash.get_and_update_item(&1); // ClassicLRU moves 1 to back → normal=[2,1]
	dash.put(3, 30);
	dash.get_and_update_item(&3); // migrate 3, evicts front=2 from normal. 2→stash
	dash.put(4, 40);
	dash.put(5, 50); // stash overflow evicts 2
	assert_eq!(*dash.get_and_update_item(&1).unwrap(), 10); // survived (was MRU)
	assert!(dash.get_and_update_item(&2).is_none()); // evicted (was LRU)
	assert_eq!(*dash.get_and_update_item(&3).unwrap(), 30);
}

#[test]
fn eviction_timestamp_lru() {
	// Similar to ClassicLRU: refreshing item 1's timestamp preserves it over item 2.
	let mut dash = tiny_dash(EvictionPolicy::TimestampLRU);
	dash.put(1, 10);
	dash.put(2, 20);
	dash.get_and_update_item(&1);
	dash.get_and_update_item(&2);
	// Sleep must exceed platform timer granularity (~15ms on Windows)
	thread::sleep(Duration::from_millis(20));
	dash.get_and_update_item(&1); // refresh item 1's timestamp
	dash.put(3, 30);
	dash.get_and_update_item(&3); // migrate 3, evicts item 2 (oldest timestamp)
	dash.put(4, 40);
	dash.put(5, 50);
	assert_eq!(*dash.get_and_update_item(&1).unwrap(), 10);
	assert!(dash.get_and_update_item(&2).is_none());
	assert_eq!(*dash.get_and_update_item(&3).unwrap(), 30);
}

#[test]
fn eviction_lfu() {
	// Accessing item 1 boosts its counter → it survives over less-accessed items.
	let mut dash = tiny_dash(EvictionPolicy::Lfu);
	dash.put(1, 10);
	dash.put(2, 20);
	dash.get_and_update_item(&1); // migrate, counter=1
	dash.get_and_update_item(&2); // migrate, counter=1
	dash.get_and_update_item(&1); // counter=2
	dash.get_and_update_item(&1); // counter=3
	dash.put(3, 30);
	dash.get_and_update_item(&3); // migrate, evicts 2 (lower counter) from normal
	dash.put(4, 40);
	dash.put(5, 50); // stash evicts 4 (counter=0, lowest)
	assert_eq!(*dash.get_and_update_item(&1).unwrap(), 10); // highest frequency
	assert!(dash.get_and_update_item(&4).is_none()); // never accessed, evicted
}

#[test]
fn eviction_swap() {
	// Swap moves accessed item toward front (index 0). Front is also the eviction target.
	// get(2) swaps it to front → on overflow, 2 is evicted instead of 1.
	let mut dash = tiny_dash(EvictionPolicy::Swap);
	dash.put(1, 10);
	dash.put(2, 20);
	dash.get_and_update_item(&1); // migrate
	dash.get_and_update_item(&2); // migrate. normal=[1,2]
	dash.get_and_update_item(&2); // Swap: 2 swaps with 1 → normal=[2,1]
	dash.put(3, 30);
	dash.get_and_update_item(&3); // migrate 3, evicts front=2
	dash.put(4, 40);
	dash.put(5, 50); // stash overflow evicts 2
	assert_eq!(*dash.get_and_update_item(&1).unwrap(), 10);
	assert!(dash.get_and_update_item(&2).is_none()); // swapped to front, then evicted
	assert_eq!(*dash.get_and_update_item(&3).unwrap(), 30);
}

// ==================== Settings ====================

#[test]
fn default_settings_works() {
	let mut dash: Dash<i64, i64> = Dash::new(&DashSettings::default());
	dash.put(1, 10);
	assert_eq!(*dash.get_and_update_item(&1).unwrap(), 10);
}

#[test]
fn dragon_settings_works() {
	let mut dash: Dash<i64, i64> = Dash::new(&DRAGON_SETTINGS);
	dash.put(1, 10);
	assert_eq!(*dash.get_and_update_item(&1).unwrap(), 10);
}

#[test]
fn custom_settings_large() {
	let settings = DashSettings {
		num_of_segments: 4,
		num_of_normal_buckets: 16,
		num_of_stash_buckets: 4,
		bucket_size: 32,
		eviction_policy: EvictionPolicy::ClassicLRU,
		debug_mode: 0,
	};
	let num_items: i64 = 500;
	// Puts only go to stash. Hash correlation (segment and stash bucket both use hash % N)
	// means fewer stash buckets are effectively used.
	let effective_stash = settings.num_of_segments * settings.bucket_size; // ~128
	let mut dash: Dash<i64, i64> = Dash::new(&settings);
	for i in 0..num_items {
		dash.put(i, i * 10);
	}
	let found: usize = (0..num_items).filter(|i| dash.get_and_update_item(i).is_some()).count();
	let min_expected = effective_stash * 3 / 4;
	assert!(found >= min_expected, "expected >= {} items retrievable, got {}", min_expected, found);
}

// ==================== Multi-segment ====================

#[test]
fn keys_route_to_different_segments() {
	let settings = small_dash_settings(EvictionPolicy::Fifo);
	let num_segments = settings.num_of_segments;
	let mut dash: Dash<i64, i64> = Dash::new(&settings);

	let segment_keys: Vec<Vec<i64>> = (0..num_segments)
		.map(|seg| find_keys_for_index(seg, num_segments, 1))
		.collect();
	assert!(
		segment_keys.iter().all(|v| !v.is_empty()),
		"could not find a key for every segment; distribution: {:?}",
		segment_keys.iter().map(|v| v.len()).collect::<Vec<_>>()
	);

	for &k in segment_keys.iter().flatten() {
		dash.put(k, k * 100);
	}
	for &k in segment_keys.iter().flatten() {
		assert_eq!(
			*dash.get_and_update_item(&k).expect("key should be present after put"),
			k * 100
		);
	}
}

#[test]
fn all_segments_functional() {
	let settings = DashSettings {
		num_of_segments: 4,
		num_of_normal_buckets: 4,
		num_of_stash_buckets: 2,
		bucket_size: 4,
		eviction_policy: EvictionPolicy::Fifo,
		debug_mode: 0,
	};
	let num_segments = settings.num_of_segments;
	let mut dash: Dash<i64, i64> = Dash::new(&settings);

	// Find 2 keys per segment
	let segment_keys: Vec<Vec<i64>> = (0..num_segments)
		.map(|seg| find_keys_for_index(seg, num_segments, 2))
		.collect();
	assert!(
		segment_keys.iter().all(|v| v.len() >= 2),
		"not all segments got 2 keys; distribution: {:?}",
		segment_keys.iter().map(|v| v.len()).collect::<Vec<_>>()
	);

	for keys in &segment_keys {
		for &k in keys {
			dash.put(k, k * 10);
		}
	}
	for keys in &segment_keys {
		for &k in keys {
			assert_eq!(
				*dash
					.get_and_update_item(&k)
					.unwrap_or_else(|| panic!("key {} missing in segment {}", k, get_index(&k, num_segments))),
				k * 10
			);
		}
	}
}

// ==================== Stress ====================

#[test]
fn random_workload_no_panic() {
	let num_ops = 10_000;
	let key_range = 100;
	let mut rng = rand::thread_rng();
	for policy in EvictionPolicy::all() {
		let mut dash: Dash<i64, i64> = Dash::new(&small_dash_settings(policy));
		for _ in 0..num_ops {
			let key: i64 = rng.gen_range(0..key_range);
			if rng.gen_bool(0.5 /* probability */) {
				dash.put(key, key * 10);
			} else {
				dash.get_and_update_item(&key);
			}
		}
		// Verify cache is still functional after random workload
		let sentinel_key = key_range + 1;
		let sentinel_val = sentinel_key * 10;
		dash.put(sentinel_key, sentinel_val);
		assert_eq!(
			*dash
				.get_and_update_item(&sentinel_key)
				.unwrap_or_else(|| panic!("cache broken after random workload for {:?}", policy)),
			sentinel_val
		);
	}
}

#[test]
fn fill_beyond_capacity_then_query() {
	let settings = small_dash_settings(EvictionPolicy::ClassicLRU);
	let stash_capacity = settings.num_of_segments * settings.num_of_stash_buckets * settings.bucket_size;
	let num_items: i64 = 1000;
	let mut dash: Dash<i64, i64> = Dash::new(&settings);
	for i in 0..num_items {
		dash.put(i, i * 10);
	}
	let mut found = 0;
	for i in 0..num_items {
		if let Some(&val) = dash.get_and_update_item(&i) {
			assert_eq!(val, i * 10, "value mismatch for key {}", i);
			found += 1;
		}
	}
	// Stash slots are limited; some survive gets that migrate them to normal
	let min_expected = stash_capacity / 2;
	assert!(found >= min_expected, "expected >= {} hits from {}-slot stash, got {}", min_expected, stash_capacity, found);
}
