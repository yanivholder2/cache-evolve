# CLAUDE.md

## Research Context

M.Sc. thesis (Yaniv Holder, advised by Prof. Roy Friedman, Technion) on mitigating the **hot set problem** in N-way associative caches using Dash-inspired bucket movement. Items can migrate between adjacent buckets to reduce contention in overloaded sets. See `research-proposal.tex` for full proposal, motivation, and initial results.

## Build

```bash
cargo build                    # build library
cargo build --release          # release build
cargo run                      # simulator (crate_type must be commented out)
cargo test                     # run tests
cargo test <name>              # single test
cargo fmt                      # format
cargo fmt --check              # check format
cargo clippy                   # lint
```

Binary vs library mode: toggle `crate_type = ["cdylib"]` line in Cargo.toml.
- **Commented out** (default): binary mode, `cargo run` works
- **Uncommented**: cdylib mode for JNI, `cargo build --release` produces .dll

Library crate name is `dash` (imports: `use dash::dash::Dash`).

## Code Conventions

- **Formatting**: hard tabs, tab_spaces=2, max_width=120 (see `.rustfmt.toml`)
- **Generic bounds**: `K: Hash + Eq + Copy + Debug, V: Eq + Copy + Debug` everywhere
- **Eviction policy**: `EvictionPolicy` is `Clone` but not `Copy` -- cloned into every bucket
- **`shared` module is private** in `lib.rs` -- not accessible to external consumers
- **No error handling**: no `Result` types, `unwrap()` everywhere
- **No concurrency**: single-threaded, no `Send`/`Sync`
- **Vec backing store**: all buckets use `Vec<Item<K,V>>`

## Architecture

Cache simulator for thesis research. Two cache implementations sharing a `Bucket` trait. Integrates with Java's Caffeine cache simulator via JNI.

### Bucket Trait (`src/shared/traits/bucket.rs`) -- Central Abstraction

All eviction logic lives in trait default methods. Implementors only provide 4 accessors:
- `get_items()` / `get_items_mut()` / `get_max_size()` / `get_eviction_policy()`

Key methods: `put`, `get`, `get_from_position` (policy-specific access), `evict_item` (policy-specific eviction)

### Eviction Policies (`src/eviction_policy.rs`)

| Policy | Index | Access behavior | Eviction target |
|--------|-------|-----------------|-----------------|
| ClassicLRU | 0 | Move to end | Remove front (index 0) |
| Lifo | 1 | No-op | Pop last |
| Lfu | 2 | Increment counter | Min counter |
| Fifo | 3 | No-op | Remove front (index 0) |
| TimestampLRU | 4 | Update timestamp | Min elapsed (BUGGED - see backlog.md) |
| Swap | 5 | Swap with prev position | Remove front (index 0) |

Index = `from_usize` mapping used by JNI.

### Dash Cache (`src/dash/`)

Segments -> Buckets (normal + stash) -> Items.
- **put**: always inserts into stash bucket (by key hash)
- **get**: checks stash first, promotes to normal bucket on hit, evicted normal items go to stash
- **Probing**: target bucket + next bucket only, no wrap-around (last bucket has no neighbor)
- No duplicate key check in `DashSegment::put`
- Config: `DashSettings` (segments, normal_buckets, stash_buckets, bucket_size, eviction_policy, debug_mode)

### Associative Cache (`src/associative_cache/`)

Buckets -> Items. Direct hash-to-bucket mapping.
- Config: `AssociativeCacheSettings` (num_of_buckets, bucket_size, eviction_policy)

### Item (`src/shared/item.rs`)

`Item<K,V>` with: key, value, lfu_counter (for LFU), timestamp (for TimestampLRU).

### JNI (`src/dash/dash_jni.rs`, `src/associative_cache/associative_cache_jni.rs`)

Bridges to Java Caffeine simulator. Types fixed to `i64`. Miss sentinel: `-1`.
- Raw pointer management via `Box::into_raw` / `Box::from_raw`
- Dash JNI logs to `rust-logs/`; Associative JNI has no logging
- Java classes: `com.github.benmanes.caffeine.cache.simulator.policy.{dash.DashRustPolicy, associative.AssociativeCacheRustPolicy}`

## Known Bugs

See **`backlog.md`** for detailed descriptions, root causes, and fix hints.

1. TimestampLRU evicts MRU instead of LRU (`bucket.rs:121-127`)
2. AssociativeCache ignores num_of_buckets (`associative_cache.rs:28-31`)
3. Swap get_from_position returns wrong item (`bucket.rs:96-100`)
4. put on existing key doesn't update value (`bucket.rs:37-41`)
5. Stale doc example in Item (`item.rs:6`)

## TODOs in Code

13 TODOs across the codebase. Key themes:
- **Performance**: Vec->LinkedList/VecDeque for O(1) eviction, `Vec::with_capacity`, fixed-size arrays
- **Ownership**: pass settings by reference, make eviction_policy a reference with lifetime
- **Architecture**: create Segment trait, blanket Display impl
- **Testing**: empty test modules in `dash_segment.rs` and `tests/integration_test.rs`

## Testing

**Current state**: 1 unit test (`utils.rs:test_get_index`). Everything else is untested.

**Priority test targets** (most impactful):
1. `Bucket` trait methods -- evict_item, get_from_position per policy
2. AssociativeCache -- multi-bucket behavior, put/get flows
3. DashSegment -- stash promotion, probing, eviction cascading
4. Edge cases -- empty/full cache, duplicate keys, eviction order

## Key File Map

| File | Purpose |
|------|---------|
| `src/lib.rs` | Library exports (3 public modules) |
| `src/main.rs` | Simulator driver (random workload) |
| `src/eviction_policy.rs` | EvictionPolicy enum (6 variants) |
| `src/shared/traits/bucket.rs` | Bucket trait (all eviction logic) |
| `src/shared/item.rs` | Item<K,V> with eviction metadata |
| `src/shared/utils.rs` | hash() and get_index() |
| `src/dash.rs` | Dash cache (segments -> buckets) |
| `src/dash/dash_segment.rs` | Segment with stash promotion logic |
| `src/dash/dash_bucket.rs` | DashBucket (Bucket trait impl) |
| `src/dash/dash_settings.rs` | DashSettings + DRAGON_SETTINGS |
| `src/dash/dash_jni.rs` | JNI bridge for Dash |
| `src/associative_cache.rs` | AssociativeCache (flat buckets) |
| `src/associative_cache/associative_cache_bucket.rs` | AssociativeCacheBucket (Bucket trait impl) |
| `src/associative_cache/associative_cache_settings.rs` | AssociativeCacheSettings |
| `src/associative_cache/associative_cache_jni.rs` | JNI bridge for AssociativeCache |
