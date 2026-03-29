# Backlog

## TOC

### Bugs

_(none)_

### Tasks

- [Review public APIs of each component](#review-public-apis-of-each-component)
- [CI: GitHub Actions for build + test on PRs](#ci-github-actions-for-build--test-on-prs)
- [Segment trait abstraction](#segment-trait-abstraction)
- [Blanket Display impl for Bucket](#blanket-display-impl-for-bucket)
- [Re-export Item publicly](#re-export-item-publicly)
- [O(1) eviction data structures](#o1-eviction-data-structures)
- [Fixed-size arrays for buckets](#fixed-size-arrays-for-buckets)
- [Eviction policy reference with lifetime](#eviction-policy-reference-with-lifetime)
- [put error on existing key](#put-error-on-existing-key)
- [Review Dash integration tests](#review-dash-integration-tests)

### Performance

- [O(1) bucket eviction via VecDeque](#o1-bucket-eviction-via-vecdeque)

### Small TODOs

_(none)_

---

## Bugs

_(none)_

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

### Review Dash integration tests
- **Created**: 2026-03-27 | **Priority**: 5
- Review `tests/dash_integration.rs` manually for correctness and completeness. Key areas:
	- Probing bucket tests (`probing_bucket_lookup`, `last_bucket_has_no_probing_neighbor`) — verify they actually exercise the probing path, not just normal lookup
	- `put_after_migration_returns_fresh_value` — confirm stash-first lookup order is the intended design, not a latent bug from duplicate keys
	- Assertion thresholds in stress tests — verify `min_expected` bounds are calibrated correctly for the cache geometry

## Performance

### O(1) bucket eviction via VecDeque
- **Created**: 2026-03-24 | **Priority**: 3
- `Vec::remove(0)` in `evict_item()` is O(n) — shifts all elements. With large buckets this becomes significant. Replace `Vec<Item>` with `VecDeque<Item>` for O(1) `pop_front()`. Requires changing the `Bucket` trait's `get_items`/`get_items_mut` return types from `&Vec` to `&VecDeque`, which ripples to both `DashBucket` and `AssociativeCacheBucket` impls.

## Small TODOs

_(none)_
