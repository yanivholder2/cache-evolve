# Backlog

## TOC

### Bugs

- [Dash uniqueness invariant unenforced](#dash-uniqueness-invariant-unenforced)
- [JNI init asserts panic across FFI](#jni-init-asserts-panic-across-ffi)

### Tasks

- [Review public APIs of each component](#review-public-apis-of-each-component)
- [CI: GitHub Actions for build + test on PRs](#ci-github-actions-for-build--test-on-prs)
- [Deduplicate dash JNI boilerplate](#deduplicate-dash-jni-boilerplate)
- [Review Dash integration tests](#review-dash-integration-tests)
- [Segment trait abstraction](#segment-trait-abstraction)
- [Blanket Display impl for Bucket](#blanket-display-impl-for-bucket)
- [Re-export Item publicly](#re-export-item-publicly)
- [O(1) eviction data structures](#o1-eviction-data-structures)
- [Fixed-size arrays for buckets](#fixed-size-arrays-for-buckets)
- [Eviction policy reference with lifetime](#eviction-policy-reference-with-lifetime)
- [put error on existing key](#put-error-on-existing-key)

### Performance

- [O(1) bucket eviction via VecDeque](#o1-bucket-eviction-via-vecdeque)
- [Faster promotion_candidate_index for LFU / TimestampLRU](#faster-promotion_candidate_index-for-lfu--timestamplru)

### Small TODOs

_(none)_

---

## Bugs

### Dash uniqueness invariant unenforced
- **Created**: 2026-04-28 | **Priority**: 1
- `Dash` doc comment (`src/dash/mod.rs:18-22`) claims "each key resides in at most one location", but `Segment::put` (default in `src/dash/segments/mod.rs:105-108`) unconditionally routes new items to stash. A re-put after a stash→target migration leaves a stash duplicate while the original sits in target; `promotion_get` returns the stash copy first and re-promotes it, leaving the old target copy stale.
- `tests/dash_integration.rs:140` (`reput_between_gets_updates_value`) currently codifies the buggy behavior — masks rather than exposes the duplicate.
- Options: (a) make `Segment::put` look up & overwrite/remove existing entries in target/probing first, or (b) revise the `Dash` doc + JNI layer to formally permit/handle duplicates. Either way, the doc and the code must agree.

### JNI init asserts panic across FFI
- **Created**: 2026-04-28 | **Priority**: 1
- `SegmentCore::new` (`src/dash/segments/mod.rs:51-52`) and `Dash::new` (`src/dash/mod.rs:40`) `assert!` on bad settings. Both are reachable from the JNI init path; a panic across FFI aborts the JVM.
- `DashSettings::from_jni_args` accepts `num_of_stash_buckets = 0` (only `< 0` is rejected) → assert fires inside the constructor.
- Fix: tighten `from_jni_args` to reject `<= 0` for both `num_of_stash_buckets` and `num_of_segments`, and convert the constructor asserts to return `Result`/`None` so JNI init can return `0` instead of aborting the JVM.

## Tasks

### Review public APIs of each component
- **Created**: 2026-03-25 | **Priority**: 1
- Audit the public API surface of each component (public functions, methods, trait impls). Goals:
	- Ensure consistent naming, parameter ordering, return types
	- Verify pub visibility is intentional (not over-exposed internals)
	- Check that each public fn has clear semantics and appropriate error handling
- Components to review: `Dash`, `DashSegment`, `DashBucket`, `AssociativeCache`, `Bucket` trait, `EvictionPolicy`, JNI layers, shared types

### CI: GitHub Actions for build + test on PRs
- **Created**: 2026-03-24 | **Priority**: 1
- Add `.github/workflows/ci.yml` that runs `cargo build` and `cargo test` on every PR. Repo is private — free tier gives 2,000 min/month.

### Deduplicate dash JNI boilerplate
- **Created**: 2026-04-28 | **Priority**: 2
- `src/dash/jni/dash_combined_jni.rs`, `dash_promotion_cold_jni.rs`, `dash_promotion_hot_jni.rs` have ~160 lines of byte-identical bodies for `getFromCacheIfPresent`, `putToCache`, `dropCache`. Only the Java class name in the symbol and `CacheTy` differ. Init paths for cold/hot differ only by the log-name string and segment type.
- Extract a `jni_cache_exports!` declarative macro keyed on `(java_class_path, cache_ty, log_name)`. Combined keeps a per-file `initCache` because of the extra `tie_breaker` arg.

### Review Dash integration tests
- **Created**: 2026-03-27 | **Priority**: 5
- Review `tests/dash_integration.rs` manually for correctness and completeness. Key areas:
	- Probing bucket tests (`probing_bucket_lookup`, `last_bucket_has_no_probing_neighbor`) — verify they actually exercise the probing path, not just normal lookup
	- `put_after_migration_returns_fresh_value` — confirm stash-first lookup order is the intended design, not a latent bug from duplicate keys
	- Assertion thresholds in stress tests — verify `min_expected` bounds are calibrated correctly for the cache geometry

### Segment trait abstraction
- **Created**: 2026-03-18 | **Priority**:
- Create `Segment` trait for segment abstraction (`dash_segment.rs:21`)

### Blanket Display impl for Bucket
- **Created**: 2026-03-18 | **Priority**:
- Blanket `Display` impl for `Bucket` trait (orphan rule issue) (`bucket.rs:153`)

### Re-export Item publicly
- **Created**: 2026-03-18 | **Priority**:
- Re-export `Item` publicly from `lib.rs` (currently inaccessible via `shared`)

### O(1) eviction data structures
- **Created**: 2026-03-18 | **Priority**:
- Vec->LinkedList/VecDeque for O(1) eviction in ClassicLRU/FIFO/Swap (`bucket.rs:117`)

### Fixed-size arrays for buckets
- **Created**: 2026-03-18 | **Priority**:
- Fixed-size arrays instead of Vec for buckets (`dash_bucket.rs:29`)

### Eviction policy reference with lifetime
- **Created**: 2026-03-18 | **Priority**:
- Make `eviction_policy` a reference with lifetime instead of cloned value (`dash_bucket.rs:18`)

### put error on existing key
- **Created**: 2026-03-18 | **Priority**:
- `put` should return error/signal when key already exists (`bucket.rs:36`)

## Performance

### O(1) bucket eviction via VecDeque
- **Created**: 2026-03-24 | **Priority**: 3
- `Vec::remove(0)` in `evict_item()` is O(n) — shifts all elements. With large buckets this becomes significant. Replace `Vec<Item>` with `VecDeque<Item>` for O(1) `pop_front()`. Requires changing the `Bucket` trait's `get_items`/`get_items_mut` return types from `&Vec` to `&VecDeque`, which ripples to both `DashBucket` and `AssociativeCacheBucket` impls.

### Faster promotion_candidate_index for LFU / TimestampLRU
- **Created**: 2026-04-20 | **Priority**: 4
- `promotion_candidate_index` (and symmetrically `eviction_candidate_index`) scans the whole bucket for LFU (max counter) and TimestampLRU (max timestamp) — O(bucket) per call. Called once per promotion-segment hit. Acceptable for now (small buckets, need hit-rate numbers first); revisit if profiling shows it hot. Options: per-bucket max-heap keyed on the policy's hotness metric, or a cached max that invalidates on counter/timestamp update. Would give O(log n) or O(1) hottest-check.

## Small TODOs

_(none)_
