// use rand::Rng;

// use dash::dash::dash_settings::DashSettings;
// use dash::dash::Dash;
// use dash::settings::EvictionPolicy;

// fn simulator() {
// 	// Simulator Settings
// 	let lower_bound = 1;
// 	let upper_bound = 30;
// 	let events_number = 100;

// 	// Dash settings
// 	let settings = DashSettings {
// 		num_of_segments: 1,
// 		num_of_normal_buckets: 2,
// 		num_of_stash_buckets: 8,
// 		bucket_size: 4,
// 		eviction_policy: EvictionPolicy::Lru,
// 		debug_mode: 2,
// 	};

// 	let mut rng = rand::thread_rng();
// 	let mut cache = Dash::new(settings);

// 	for _ in 0..events_number {
// 		let number = rng.gen_range(lower_bound..=upper_bound);
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
