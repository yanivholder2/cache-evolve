# CLAUDE.md

## Research Context

M.Sc. thesis (Yaniv Holder, advised by Prof. Roy Friedman, Technion) on mitigating the **hot set problem** in N-way associative caches using Dash-inspired bucket movement. Items can migrate between adjacent buckets to reduce contention in overloaded sets. See `research-proposal.tex` for full proposal, motivation, and initial results.

## Git

Remote repo: `yanivholder2/dash-cache-rs` (private, GitHub). Push every new commit.

## Build

```bash
cargo build                    # build library
cargo build --release          # release build
cargo run                      # simulator
cargo test                     # run tests
cargo test <name>              # single test
cargo fmt                      # format
cargo fmt --check              # check format
cargo clippy                   # lint
```

Both `cdylib` (for JNI .dll) and `lib` (for binary/tests) are always built — no toggling needed.

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
| TimestampLRU | 1 | Update timestamp | Oldest timestamp (min Instant) |
| Lifo | 2 | No-op | Pop last |
| Lfu | 3 | Increment counter | Min counter |
| Fifo | 4 | No-op | Remove front (index 0) |
| Swap | 5 | Swap with prev position | Remove front (index 0) |

Index = `from_usize` mapping used by JNI. Matches enum declaration order and Java `DashRustEvictionPolicy` ordinals.

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

## Caffeine Simulator Integration

Dash cache integrates with [Caffeine](https://github.com/ben-manes/caffeine)'s cache simulator via JNI.

**Repos**:
- Caffeine (active): `C:\Users\yanivholder\OneDrive - Microsoft\Desktop\Personal\Thesis\caffeine`
- Caffeine (reference, known-working): `C:\Users\yanivholder\OneDrive - Microsoft\Desktop\Personal\Thesis\caffeine-old`

**Java policy class**: `simulator/src/main/java/.../policy/dash/DashRustPolicy.java`
- Uses `policies(Config)` factory method (registered via `registerMany` in `Registry.java`)
- Pointer-based JNI: stores `cachePointer` from `initDefaultCache()` / `initCache(...)`, passes to all calls
- `DashRustEvictionPolicy` enum ordinals must match Rust `from_usize` indices
- `DashSettings` inner class reads `dash.*` config from `application.conf`

**Config**: `simulator/src/main/resources/application.conf`
- `dash.numOfNormalBuckets`, `dash.numOfStashBuckets`, `dash.bucketSize`, `dash.debugMode`
- `dash.policy` -- list of eviction policy names (e.g., `[classiclru, lfu]`)
- `maximum-size` -- total cache capacity (segments computed from this)
- `policies` list must include `dash.DashRust`

**Build + Run workflow**:
1. `cargo build --release` (produces `target/release/dash.dll`)
2. From caffeine dir: `./gradlew simulator:run -PjvmArgs="-XX:+UseParallelGC,-Xmx8g,-Djava.library.path=<path-to-dash-cache-rs>/target/release" -q`

**AssociativeCache**: Java side (`AssociativeCacheRustPolicy.java`) exists in caffeine-old but not yet in caffeine. See backlog.

## Backlog (`backlog.md`)

Track bugs, tasks, and small TODOs in `backlog.md`. When discovering issues, missing features, or improvement opportunities during work, add them to the appropriate section.

**Sections**: Bugs, Tasks, Small TODOs. Each item gets its own `###` heading under the section.

**Format per item**:
```markdown
### Item title
- **Created**: YYYY-MM-DD | **Priority**: (0-5, 0 = most important, leave empty if unknown)
- Description with file references (`file.rs:line`)
```

**TOC**: Every item must be linked in the TOC at the top, grouped under its section.

**When to add**: bugs found but not fixed now, refactoring opportunities, missing tests, performance improvements, API issues.

**When to remove**: after an item is fully resolved, remove it and its TOC entry.

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
