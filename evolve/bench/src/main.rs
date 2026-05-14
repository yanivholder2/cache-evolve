/// Microbench harness — std-only, single-threaded.
/// Measures p50/p99 ns for get and put on a stub LRU cache, then writes bench.json.
///
/// Usage: dash-bench [--output <path>]   (default: bench.json in current dir)
use std::{
    collections::VecDeque,
    env,
    fs,
    time::{Duration, Instant},
};

const WARMUP: usize = 500;
const SAMPLES: usize = 2_000;
const CACHE_CAP: usize = 64;
const KEY_RANGE: u64 = 128;

// ── stub LRU cache ────────────────────────────────────────────────────────────

struct LruCache {
    cap: usize,
    order: VecDeque<u64>,
    store: std::collections::HashMap<u64, u64>,
}

impl LruCache {
    fn new(cap: usize) -> Self {
        Self { cap, order: VecDeque::with_capacity(cap + 1), store: std::collections::HashMap::new() }
    }

    fn get(&mut self, key: u64) -> Option<u64> {
        if let Some(&v) = self.store.get(&key) {
            self.order.retain(|&k| k != key);
            self.order.push_back(key);
            Some(v)
        } else {
            None
        }
    }

    fn put(&mut self, key: u64, value: u64) {
        if self.store.contains_key(&key) {
            self.order.retain(|&k| k != key);
        } else if self.store.len() >= self.cap {
            if let Some(evicted) = self.order.pop_front() {
                self.store.remove(&evicted);
            }
        }
        self.store.insert(key, value);
        self.order.push_back(key);
    }
}

// ── measurement helpers ───────────────────────────────────────────────────────

fn percentile(samples: &mut Vec<Duration>, pct: f64) -> u64 {
    samples.sort_unstable();
    let idx = ((samples.len() as f64 * pct / 100.0) as usize).min(samples.len() - 1);
    samples[idx].as_nanos() as u64
}

fn measure<F: FnMut()>(mut f: F) -> Vec<Duration> {
    // warmup
    for _ in 0..WARMUP {
        f();
    }
    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        let t = Instant::now();
        f();
        samples.push(t.elapsed());
    }
    samples
}

// ── simple deterministic key sequence ────────────────────────────────────────

fn next_key(state: &mut u64) -> u64 {
    // xorshift64
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state % KEY_RANGE + 1
}

// ── main ──────────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = env::args().collect();
    let output = args.windows(2)
        .find(|w| w[0] == "--output")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| "bench.json".to_string());

    let mut cache = LruCache::new(CACHE_CAP);
    let mut rng: u64 = 0xdeadbeef_cafebabe;

    // pre-fill
    for i in 0..CACHE_CAP / 2 {
        cache.put(i as u64 + 1, i as u64);
    }

    let get_samples = measure(|| {
        let k = next_key(&mut rng);
        cache.get(k);
    });

    let put_samples = measure(|| {
        let k = next_key(&mut rng);
        cache.put(k, k * 2);
    });

    let mut get_samples = get_samples;
    let mut put_samples = put_samples;
    let get_p50 = percentile(&mut get_samples, 50.0);
    let get_p99 = percentile(&mut get_samples, 99.0);
    let put_p50 = percentile(&mut put_samples, 50.0);
    let put_p99 = percentile(&mut put_samples, 99.0);

    // summary = get p50 (most representative single-op latency)
    let summary = get_p50;

    let json = format!(
        r#"{{
  "dim": "per-op-time",
  "values": [
    {{"op": "get", "p50_ns": {get_p50}, "p99_ns": {get_p99}}},
    {{"op": "put", "p50_ns": {put_p50}, "p99_ns": {put_p99}}}
  ],
  "summary": {summary},
  "higher_is_better": false
}}
"#,
        get_p50 = get_p50,
        get_p99 = get_p99,
        put_p50 = put_p50,
        put_p99 = put_p99,
        summary = summary,
    );

    fs::write(&output, &json).unwrap_or_else(|e| {
        eprintln!("error writing {}: {}", output, e);
        std::process::exit(1);
    });
    print!("{}", json);
}
