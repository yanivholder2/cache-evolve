// use dash::associative_cache::{associative_cache_settings::AssociativeCacheSettings, AssociativeCache};
// use dash::dash::dash_settings::DashSettings;
// use dash::dash::Dash;
// use dash::eviction_policy::EvictionPolicy;

// use rand::Rng;

// // Simulator Settings
// const LOWER_BOUND: usize = 1;
// const UPPER_BOUND: usize = 30;
// const EVENTS_NUMBER: usize = 100;

// fn simulator() {
// 	// Assosiative Cache settings
// 	let settings = AssociativeCacheSettings {
// 		num_of_buckets: 1,
// 		bucket_size: 8,
// 		eviction_policy: EvictionPolicy::ClassicLRU,
// 	};
// 	let mut cache = AssociativeCache::new(settings);

// 	// // Dash settings
// 	// let settings = DashSettings {
// 	// 	num_of_segments: 1,
// 	// 	num_of_normal_buckets: 2,
// 	// 	num_of_stash_buckets: 8,
// 	// 	bucket_size: 4,
// 	// 	eviction_policy: EvictionPolicy::Lru,
// 	// 	debug_mode: 2,
// 	// };
// 	// let mut cache = Dash::new(settings);

// 	let mut rng = rand::thread_rng();

// 	for _ in 0..EVENTS_NUMBER {
// 		let number = rng.gen_range(LOWER_BOUND..=UPPER_BOUND);
// 		let res = cache.get_and_update_item(&number);
// 		match res {
// 			None => {
// 				cache.put(number, number);
// 				println!("Cache miss for number: {}", number);
// 			}
// 			Some(value) => {
// 				println!("Cache hit for number: {}", value);
// 			}
// 		}
// 		println!("----------------------------------------------------------------------------------------------------");
// 		println!("{}", cache);
// 		println!("----------------------------------------------------------------------------------------------------");
// 	}
// }

fn main() {
	// simulator();
}
