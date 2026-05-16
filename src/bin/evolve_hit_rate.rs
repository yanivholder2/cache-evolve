//! Per-configuration hit-rate measurement for fitness plugin consumption.
//!
//! Runs a fixed-seed synthetic workload and emits one plugin-shape JSON record:
//! `{ "dim": "hit-rate", "values": [{"config","hits","misses","hit_rate"}], "summary": <hit_rate>, "higher_is_better": true }`.
//!
//! The `hit-rate.py` plugin invokes this binary once per (cache, size) configuration
//! and merges the per-invocation records into the final plugin output.
//!
//! Usage: `evolve-hit-rate [--cache <dash|assoc>] [--events <n>] [--seed <n>] [--size <n>]`.
use dash::associative_cache::{associative_cache_settings::AssociativeCacheSettings, AssociativeCache};
use dash::dash::dash_settings::DashSettings;
use dash::dash::segments::PromotionColdSegment;
use dash::dash::Dash;
use dash::fitness_json::write_plugin_json;
use dash::shared::eviction_policy::EvictionPolicy;
use std::env;
use std::str::FromStr;

const DEFAULT_EVENTS: usize = 2000;
const DEFAULT_SEED: u64 = 42;
const DEFAULT_SIZE: usize = 32;

/// Workload key range expressed as a multiple of cache capacity — wide enough to force a mix of hits/misses.
const KEY_RANGE_FACTOR: usize = 3;

/// Minimum `--size` so derived bucket counts (`size`, `size / STASH_DIVISOR`) are both ≥ 2 without clamping.
const MIN_SIZE: usize = 8;

const BUCKET_SIZE: usize = 4;
const STASH_DIVISOR: usize = 4;

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

fn parse_or_die<T: FromStr>(flag: &str, raw: &str) -> T
where
    T::Err: std::fmt::Display,
{
    raw.parse().unwrap_or_else(|e| {
        eprintln!("error: {flag} expected a number, got {raw:?}: {e}");
        std::process::exit(2);
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let get_arg = |flag: &str| -> Option<String> {
        args.iter()
            .position(|a| a == flag)
            .map(|i| {
                args.get(i + 1).cloned().unwrap_or_else(|| {
                    eprintln!("error: {flag} requires a value");
                    std::process::exit(2);
                })
            })
    };

    let cache_type = get_arg("--cache").unwrap_or_else(|| "assoc".to_string());
    let events: usize = get_arg("--events").as_deref().map(|s| parse_or_die("--events", s)).unwrap_or(DEFAULT_EVENTS);
    let seed: u64 = get_arg("--seed").as_deref().map(|s| parse_or_die("--seed", s)).unwrap_or(DEFAULT_SEED);
    let size: usize = get_arg("--size").as_deref().map(|s| parse_or_die("--size", s)).unwrap_or(DEFAULT_SIZE);

    if cache_type != "assoc" && cache_type != "dash" {
        eprintln!("error: --cache must be 'assoc' or 'dash', got {cache_type:?}");
        std::process::exit(2);
    }
    if events == 0 {
        eprintln!("error: --events must be > 0");
        std::process::exit(2);
    }
    if size < MIN_SIZE {
        eprintln!("error: --size must be ≥ {MIN_SIZE}, got {size}");
        std::process::exit(2);
    }
    let key_range: u64 = (size * KEY_RANGE_FACTOR) as u64;

    let (hits, misses) = match cache_type.as_str() {
        "dash" => {
            let s = DashSettings {
                num_of_segments: 1,
                num_of_normal_buckets: size,
                num_of_stash_buckets: size / STASH_DIVISOR,
                bucket_size: BUCKET_SIZE,
                eviction_policy: EvictionPolicy::ClassicLRU,
                debug_mode: 0,
            };
            run(Dash::new(s.num_of_segments, || PromotionColdSegment::new(&s)), events, seed, key_range)
        }
        _ => {
            run(AssociativeCache::new(AssociativeCacheSettings {
                num_of_buckets: size,
                bucket_size: BUCKET_SIZE,
                eviction_policy: EvictionPolicy::ClassicLRU,
            }), events, seed, key_range)
        }
    };

    let hit_rate = hits as f64 / (hits + misses) as f64;
    let label = format!("{cache_type}-size{size}");
    let values = format!(
        r#"[{{"config":"{label}","hits":{hits},"misses":{misses},"hit_rate":{hit_rate:.6}}}]"#,
    );
    println!("{}", write_plugin_json("hit-rate", &values, format!("{hit_rate:.6}"), true));
}
