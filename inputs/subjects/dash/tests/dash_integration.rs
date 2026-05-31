use dash::dash::dash_settings::DashSettings;
use dash::dash::segments::DashSegment;
use dash::dash::Dash;
use dash::shared::eviction_policy::EvictionPolicyKind;

fn probing_settings(policy: EvictionPolicyKind) -> DashSettings {
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
fn dash_segment_put_then_get() {
	let settings = probing_settings(EvictionPolicyKind::ClassicLRU);
	let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || DashSegment::new(&settings));
	cache.put(1, 10);
	assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
}

#[test]
fn dash_segment_many_keys() {
	let settings = probing_settings(EvictionPolicyKind::ClassicLRU);
	let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || DashSegment::new(&settings));
	for i in 0..20 {
		cache.put(i, i * 10);
	}
	let hits: usize = (0..20).filter(|i| cache.get_and_update_item(i).is_some()).count();
	assert!(hits > 0);
}

#[test]
fn dash_segment_repeated_access_promotes() {
	let settings = probing_settings(EvictionPolicyKind::ClassicLRU);
	let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || DashSegment::new(&settings));
	cache.put(1, 10);
	for _ in 0..10 {
		assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
	}
}

#[test]
fn dash_segment_miss() {
	let settings = probing_settings(EvictionPolicyKind::ClassicLRU);
	let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || DashSegment::new(&settings));
	assert!(cache.get_and_update_item(&999).is_none());
}

#[test]
fn random_workload_no_panic() {
	use rand::Rng;
	let mut rng = rand::thread_rng();
	let s_lru = probing_settings(EvictionPolicyKind::ClassicLRU);

	let mut cache: Dash<i64, i64, _> = Dash::new(s_lru.num_of_segments, || DashSegment::new(&s_lru));

	const NUM_OPS: usize = 1000;
	const KEY_SPACE: i64 = 50;
	const PUT_PROBABILITY: f64 = 0.5;
	for _ in 0..NUM_OPS {
		let key: i64 = rng.gen_range(0..KEY_SPACE);
		if rng.gen_bool(PUT_PROBABILITY) {
			// Mirror the Caffeine call contract: only put on a miss.
			if cache.get_and_update_item(&key).is_none() {
				cache.put(key, key);
			}
		} else {
			cache.get_and_update_item(&key);
		}
	}
}
