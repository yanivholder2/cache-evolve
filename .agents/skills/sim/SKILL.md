---
name: sim
description: Configure and run the standalone Rust cache simulator (cargo run, not Caffeine JNI). Use when the user wants to run the Rust binary directly, test cache behavior locally, or change simulator settings in main.rs. Triggers on "cargo run", "run standalone", "run main".
argument-hint: "[cache-type] [eviction-policy]"
model: opus
allowed-tools: Bash(cargo run*), Bash(cargo fmt*), Read, Edit
---

Configure and run the cache simulator.

$ARGUMENTS format: `<cache_type> <eviction_policy> [size_params...]`
- cache_type: `dash` or `associative`
- eviction_policy: ClassicLRU, TimestampLRU, Fifo, Lifo, Lfu, Swap
- size_params: optional, depends on cache type

Steps:
1. Update `src/main.rs` with the requested cache type, policy, and size params (toggle `USE_DASH` and settings)
2. Run `cargo run` and show output
3. If output is large, summarize hit/miss statistics
