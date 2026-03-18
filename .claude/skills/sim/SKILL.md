---
description: Configure and run the cache simulator
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
1. Ensure `crate-type = ["cdylib"]` is **commented out** in Cargo.toml
2. Update `src/main.rs` `simulator()` with the requested cache type, policy, and size params
3. Run `cargo run` and show output
4. If output is large, summarize hit/miss statistics
