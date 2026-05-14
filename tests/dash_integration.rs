use dash::dash::dash_settings::DashSettings;
use dash::dash::segments::*;
use dash::dash::Dash;
use dash::shared::eviction_policy::EvictionPolicy;

fn probing_settings(policy: EvictionPolicy) -> DashSettings {
	DashSettings {
		num_of_segments: 2,
		num_of_normal_buckets: 4,
		num_of_stash_buckets: 2,
		bucket_size: 4,
		eviction_policy: policy,
		debug_mode: 0,
	}
}

#[test]
fn promotion_cold_put_then_get() {
	let settings = probing_settings(EvictionPolicy::ClassicLRU);
	let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || PromotionColdSegment::new(&settings));
	cache.put(1, 10);
	assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
}

#[test]
fn promotion_cold_many_keys() {
	let settings = probing_settings(EvictionPolicy::ClassicLRU);
	let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || PromotionColdSegment::new(&settings));
	for i in 0..20 {
		cache.put(i, i * 10);
	}
	let hits: usize = (0..20).filter(|i| cache.get_and_update_item(i).is_some()).count();
	assert!(hits > 0);
}

#[test]
fn promotion_cold_repeated_access_promotes() {
	let settings = probing_settings(EvictionPolicy::ClassicLRU);
	let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || PromotionColdSegment::new(&settings));
	cache.put(1, 10);
	for _ in 0..10 {
		assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
	}
}

#[test]
fn promotion_cold_miss() {
	let settings = probing_settings(EvictionPolicy::ClassicLRU);
	let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || PromotionColdSegment::new(&settings));
	assert!(cache.get_and_update_item(&999).is_none());
}

#[test]
fn promotion_hot_put_then_get() {
	let settings = probing_settings(EvictionPolicy::ClassicLRU);
	let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || PromotionHotSegment::new(&settings));
	cache.put(1, 10);
	assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
}

#[test]
fn promotion_hot_many_keys() {
	let settings = probing_settings(EvictionPolicy::ClassicLRU);
	let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || PromotionHotSegment::new(&settings));
	for i in 0..20 {
		cache.put(i, i * 10);
	}
	let hits: usize = (0..20).filter(|i| cache.get_and_update_item(i).is_some()).count();
	assert!(hits > 0);
}

#[test]
fn combined_put_then_get_timestamp_lru() {
	let settings = probing_settings(EvictionPolicy::TimestampLRU);
	let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || {
		CombinedSegment::new(&settings, EvictionTieBreaker::EvictProbing)
	});
	cache.put(1, 10);
	assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
}

#[test]
fn combined_put_then_get_lfu() {
	let settings = probing_settings(EvictionPolicy::Lfu);
	let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || {
		CombinedSegment::new(&settings, EvictionTieBreaker::EvictProbing)
	});
	cache.put(1, 10);
	assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
}

#[test]
fn combined_many_keys() {
	let settings = probing_settings(EvictionPolicy::TimestampLRU);
	let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || {
		CombinedSegment::new(&settings, EvictionTieBreaker::EvictProbing)
	});
	for i in 0..20 {
		cache.put(i, i * 10);
	}
	let hits: usize = (0..20).filter(|i| cache.get_and_update_item(i).is_some()).count();
	assert!(hits > 0);
}

#[test]
fn all_variants_put_then_get() {
	let s_lru = probing_settings(EvictionPolicy::ClassicLRU);
	let s_ts = probing_settings(EvictionPolicy::TimestampLRU);

	let mut cold: Dash<i64, i64, _> = Dash::new(s_lru.num_of_segments, || PromotionColdSegment::new(&s_lru));
	let mut hot: Dash<i64, i64, _> = Dash::new(s_lru.num_of_segments, || PromotionHotSegment::new(&s_lru));
	let mut lazy: Dash<i64, i64, _> = Dash::new(s_ts.num_of_segments, || {
		CombinedSegment::new(&s_ts, EvictionTieBreaker::EvictProbing)
	});

	cold.put(1, 10);
	assert_eq!(*cold.get_and_update_item(&1).unwrap(), 10);

	hot.put(1, 10);
	assert_eq!(*hot.get_and_update_item(&1).unwrap(), 10);

	lazy.put(1, 10);
	assert_eq!(*lazy.get_and_update_item(&1).unwrap(), 10);
}

#[test]
fn all_variants_repeated_get_returns_value() {
	let s_lru = probing_settings(EvictionPolicy::ClassicLRU);
	let s_ts = probing_settings(EvictionPolicy::TimestampLRU);

	let mut cold: Dash<i64, i64, _> = Dash::new(s_lru.num_of_segments, || PromotionColdSegment::new(&s_lru));
	let mut hot: Dash<i64, i64, _> = Dash::new(s_lru.num_of_segments, || PromotionHotSegment::new(&s_lru));
	let mut lazy: Dash<i64, i64, _> = Dash::new(s_ts.num_of_segments, || {
		CombinedSegment::new(&s_ts, EvictionTieBreaker::EvictProbing)
	});

	// Promotions must not corrupt the stored value.
	cold.put(1, 10);
	assert_eq!(*cold.get_and_update_item(&1).unwrap(), 10);
	assert_eq!(*cold.get_and_update_item(&1).unwrap(), 10);

	hot.put(1, 10);
	assert_eq!(*hot.get_and_update_item(&1).unwrap(), 10);
	assert_eq!(*hot.get_and_update_item(&1).unwrap(), 10);

	lazy.put(1, 10);
	assert_eq!(*lazy.get_and_update_item(&1).unwrap(), 10);
	assert_eq!(*lazy.get_and_update_item(&1).unwrap(), 10);
}

#[test]
fn random_workload_no_panic() {
	use rand::Rng;
	let mut rng = rand::thread_rng();
	let s_lru = probing_settings(EvictionPolicy::ClassicLRU);
	let s_lfu = probing_settings(EvictionPolicy::Lfu);

	let mut cold: Dash<i64, i64, _> = Dash::new(s_lru.num_of_segments, || PromotionColdSegment::new(&s_lru));
	let mut combined: Dash<i64, i64, _> = Dash::new(s_lfu.num_of_segments, || {
		CombinedSegment::new(&s_lfu, EvictionTieBreaker::EvictProbing)
	});

	const NUM_OPS: usize = 1000;
	const KEY_SPACE: i64 = 50;
	const PUT_PROBABILITY: f64 = 0.5;
	for _ in 0..NUM_OPS {
		let key: i64 = rng.gen_range(0..KEY_SPACE);
		if rng.gen_bool(PUT_PROBABILITY) {
			// Mirror the Caffeine call contract: only put on a miss.
			if cold.get_and_update_item(&key).is_none() {
				cold.put(key, key);
			}
			if combined.get_and_update_item(&key).is_none() {
				combined.put(key, key);
			}
		} else {
			cold.get_and_update_item(&key);
			combined.get_and_update_item(&key);
		}
	}
}
