/// Measure hit rate for the Dash / AssociativeCache implementations.
///
/// Runs a fixed-seed synthetic workload and reports per-configuration hit rates as JSON.
/// Usage: evolve-hit-rate [--cache <dash|assoc>] [--events <n>] [--seed <n>] [--size <n>]
use dash::associative_cache::{associative_cache_settings::AssociativeCacheSettings, AssociativeCache};
use dash::dash::dash_settings::DashSettings;
use dash::dash::segments::PromotionColdSegment;
use dash::dash::Dash;
use dash::shared::eviction_policy::EvictionPolicy;
use std::env;

trait Cache {
    fn put(&mut self, key: u64, value: u64);
    fn get(&mut self, key: &u64) -> Option<u64>;
}

impl Cache for AssociativeCache<u64, u64> {
    fn put(&mut self, key: u64, value: u64) { AssociativeCache::put(self, key, value); }
    fn get(&mut self, key: &u64) -> Option<u64> { AssociativeCache::get_and_update_item(self, key).copied() }
}

impl Cache for Dash<u64, u64, PromotionColdSegment<u64, u64>> {
    fn put(&mut self, key: u64, value: u64) { Dash::put(self, key, value); }
    fn get(&mut self, key: &u64) -> Option<u64> { Dash::get_and_update_item(self, key).copied() }
}

fn next_key(state: &mut u64, range: u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state % range + 1
}

fn run<C: Cache>(mut cache: C, events: usize, seed: u64, key_range: u64) -> (usize, usize) {
    let mut rng = seed;
    let (mut hits, mut misses) = (0usize, 0usize);
    for _ in 0..events {
        let key = next_key(&mut rng, key_range);
        match cache.get(&key) {
            Some(_) => hits += 1,
            None => {
                misses += 1;
                cache.put(key, key);
            }
        }
    }
    (hits, misses)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let get_arg = |flag: &str, default: &str| -> String {
        args.windows(2)
            .find(|w| w[0] == flag)
            .map(|w| w[1].clone())
            .unwrap_or_else(|| default.to_string())
    };

    let cache_type = get_arg("--cache", "assoc");
    let events: usize = get_arg("--events", "2000").parse().unwrap_or(2000);
    let seed: u64 = get_arg("--seed", "42").parse().unwrap_or(42);
    let size: usize = get_arg("--size", "32").parse().unwrap_or(32);
    let key_range: u64 = (size * 3) as u64; // intentionally larger than cache for realism

    let (hits, misses) = match cache_type.as_str() {
        "dash" => {
            let s = DashSettings {
                num_of_segments: 1,
                num_of_normal_buckets: size.max(2),
                num_of_stash_buckets: (size / 4).max(2),
                bucket_size: 4,
                eviction_policy: EvictionPolicy::ClassicLRU,
                debug_mode: 0,
            };
            run(Dash::new(s.num_of_segments, || PromotionColdSegment::new(&s)), events, seed, key_range)
        }
        _ => {
            run(AssociativeCache::new(AssociativeCacheSettings {
                num_of_buckets: size.max(1),
                bucket_size: 4,
                eviction_policy: EvictionPolicy::ClassicLRU,
            }), events, seed, key_range)
        }
    };

    let hit_rate = hits as f64 / (hits + misses) as f64;
    let label = format!("{}-size{}", cache_type, size);
    println!(
        r#"{{"dim":"hit-rate","values":[{{"config":"{label}","hits":{hits},"misses":{misses},"hit_rate":{hit_rate:.6}}}],"summary":{hit_rate:.6},"higher_is_better":true}}"#,
    );
}
