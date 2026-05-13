use dash::associative_cache::{associative_cache_settings::AssociativeCacheSettings, AssociativeCache};
use dash::dash::dash_settings::DashSettings;
use dash::dash::segments::{CombinedSegment, EvictionTieBreaker, PromotionColdSegment, PromotionHotSegment, Segment};
use dash::dash::Dash;
use dash::shared::eviction_policy::EvictionPolicy;
use log::info;
use rand::Rng;

const LOWER_BOUND: usize = 1;
const UPPER_BOUND: usize = 30;
const EVENTS_NUMBER: usize = 100;

/// Toggle the simulator's cache implementation by editing this const.
const CACHE_KIND: CacheKind = CacheKind::Associative;

#[derive(Clone, Copy)]
// Only one variant is selected at a time via CACHE_KIND, so the others read as dead.
#[allow(dead_code)]
enum CacheKind {
	Associative,
	DashPromotionCold,
	DashPromotionHot,
	DashCombined,
}

fn run_simulator<C: Cache>(mut cache: C) {
	let mut rng = rand::thread_rng();

	for _ in 0..EVENTS_NUMBER {
		let number = rng.gen_range(LOWER_BOUND..=UPPER_BOUND);
		match cache.get_and_update_item(&number) {
			None => {
				cache.put(number, number);
				info!("simulator: miss key={number}");
			}
			Some(value) => {
				info!("simulator: hit key={number} value={value}");
			}
		}
	}
}

trait Cache {
	fn put(&mut self, key: usize, value: usize);
	fn get_and_update_item(&mut self, key: &usize) -> Option<usize>;
}

impl Cache for AssociativeCache<usize, usize> {
	fn put(&mut self, key: usize, value: usize) {
		AssociativeCache::put(self, key, value);
	}
	fn get_and_update_item(&mut self, key: &usize) -> Option<usize> {
		AssociativeCache::get_and_update_item(self, key).copied()
	}
}

impl<S: Segment<usize, usize>> Cache for Dash<usize, usize, S> {
	fn put(&mut self, key: usize, value: usize) {
		Dash::put(self, key, value);
	}
	fn get_and_update_item(&mut self, key: &usize) -> Option<usize> {
		Dash::get_and_update_item(self, key).copied()
	}
}

/// Promotion segments accept any policy; ClassicLRU keeps the simulator output readable.
fn promotion_settings() -> DashSettings {
	DashSettings {
		num_of_segments: 1,
		num_of_normal_buckets: 2,
		num_of_stash_buckets: 8,
		bucket_size: 4,
		eviction_policy: EvictionPolicy::ClassicLRU,
		debug_mode: 2,
	}
}

/// Combined segments require a comparable staleness metric (TimestampLRU or Lfu).
fn combined_settings() -> DashSettings {
	DashSettings {
		num_of_segments: 1,
		num_of_normal_buckets: 2,
		num_of_stash_buckets: 8,
		bucket_size: 4,
		eviction_policy: EvictionPolicy::Lfu,
		debug_mode: 2,
	}
}

fn main() {
	match CACHE_KIND {
		CacheKind::Associative => {
			dash::init_file_and_stdout_logger("associative", 2);
			run_simulator(AssociativeCache::new(AssociativeCacheSettings {
				num_of_buckets: 1,
				bucket_size: 8,
				eviction_policy: EvictionPolicy::Swap,
			}));
		}
		CacheKind::DashPromotionCold => {
			dash::init_file_and_stdout_logger("dash_promotion_cold", 2);
			let s = promotion_settings();
			run_simulator(Dash::new(s.num_of_segments, || PromotionColdSegment::new(&s)));
		}
		CacheKind::DashPromotionHot => {
			dash::init_file_and_stdout_logger("dash_promotion_hot", 2);
			let s = promotion_settings();
			run_simulator(Dash::new(s.num_of_segments, || PromotionHotSegment::new(&s)));
		}
		CacheKind::DashCombined => {
			dash::init_file_and_stdout_logger("dash_combined", 2);
			let s = combined_settings();
			run_simulator(Dash::new(s.num_of_segments, || {
				CombinedSegment::new(&s, EvictionTieBreaker::EvictProbing)
			}));
		}
	}
}
