use dash::associative_cache::{associative_cache_settings::AssociativeCacheSettings, AssociativeCache};
use dash::dash::{dash_settings::DashSettings, Dash};
use dash::eviction_policy::EvictionPolicy;
use log::{debug, info};
use rand::Rng;
use std::fmt::Display;

// Simulator Settings
const LOWER_BOUND: usize = 1;
const UPPER_BOUND: usize = 30;
const EVENTS_NUMBER: usize = 100;

// Toggle: comment/uncomment to switch cache type
const USE_DASH: bool = false;

fn run_simulator<C: Cache>(mut cache: C) {
	let mut rng = rand::thread_rng();

	for _ in 0..EVENTS_NUMBER {
		let number = rng.gen_range(LOWER_BOUND..=UPPER_BOUND);
		let res = cache.get_and_update_item(&number);
		match res {
			None => {
				cache.put(number, number);
				info!("simulator: miss key={number}");
			}
			Some(value) => {
				info!("simulator: hit key={number} value={value}");
			}
		}
		debug!("{cache}");
	}
}

trait Cache: Display {
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

impl Cache for Dash<usize, usize> {
	fn put(&mut self, key: usize, value: usize) {
		Dash::put(self, key, value);
	}
	fn get_and_update_item(&mut self, key: &usize) -> Option<usize> {
		Dash::get_and_update_item(self, key).copied()
	}
}

fn main() {
	if USE_DASH {
		dash::init_file_and_stdout_logger("dash", 2 /* debug_mode */);
		let settings = DashSettings {
			num_of_segments: 1,
			num_of_normal_buckets: 2,
			num_of_stash_buckets: 8,
			bucket_size: 4,
			eviction_policy: EvictionPolicy::ClassicLRU,
			debug_mode: 2,
		};
		run_simulator(Dash::new(&settings));
	} else {
		dash::init_file_and_stdout_logger("associative", 2 /* debug_mode */);
		let settings = AssociativeCacheSettings {
			num_of_buckets: 1,
			bucket_size: 8,
			eviction_policy: EvictionPolicy::Swap,
		};
		run_simulator(AssociativeCache::new(settings));
	}
}
