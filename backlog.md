# Backlog

## TOC

### Bugs

_(none)_

### Tasks

- [Add logging for full cache flow](#add-logging-for-full-cache-flow)
- [Segment trait abstraction](#segment-trait-abstraction)
- [Blanket Display impl for Bucket](#blanket-display-impl-for-bucket)
- [Re-export Item publicly](#re-export-item-publicly)
- [O(1) eviction data structures](#o1-eviction-data-structures)
- [Fixed-size arrays for buckets](#fixed-size-arrays-for-buckets)
- [Pass settings by reference](#pass-settings-by-reference)
- [Eviction policy reference with lifetime](#eviction-policy-reference-with-lifetime)
- [put error on existing key](#put-error-on-existing-key)
- [Bucket trait tests](#bucket-trait-tests)
- [AssociativeCache tests](#associativecache-tests)
- [DashSegment tests](#dashsegment-tests)
- [Edge case tests](#edge-case-tests)
- [Integration tests](#integration-tests)

### Small TODOs

- [Pre-allocate bucket storage](#pre-allocate-bucket-storage)
- [Derive Copy on EvictionPolicy](#derive-copy-on-evictionpolicy)
- [Share skills between agents](#share-skills-between-agents)

---

## Bugs

_(none)_

## Tasks

### Add logging for full cache flow
- **Created**: 2026-03-23 | **Priority**: 1
- Add `log` statements across the entire Dash and AssociativeCache flow for research observability. Key areas:
	- **Evictions**: stash eviction on insert (`dash_segment.rs:104`), stash eviction during promotion cascade (`dash_segment.rs:66`), normal bucket eviction (`bucket.rs:evict_item`)
	- **Promotions**: stash→normal promotion (`dash_segment.rs:57-68`)
	- **Put/Get routing**: segment selection, bucket selection, probing hits/misses
	- **AssociativeCache JNI**: add `simplelog` init (currently missing — parity with Dash JNI `dash_jni.rs:14-42`)
- Extract shared logger init from `dash_jni.rs` into a common module so both JNI bridges reuse it
- Use `trace!` for per-operation logs (high volume), `debug!` for evictions/promotions, `info!` for cache init/config

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

### Pass settings by reference
- **Created**: 2026-03-18 | **Priority**:
- Pass settings by reference instead of cloning (`dash.rs:35`, `dash_segment.rs:30`)

### Eviction policy reference with lifetime
- **Created**: 2026-03-18 | **Priority**:
- Make `eviction_policy` a reference with lifetime instead of cloned value (`dash_bucket.rs:18`)

### put error on existing key
- **Created**: 2026-03-18 | **Priority**:
- `put` should return error/signal when key already exists (`bucket.rs:36`)

### Bucket trait tests
- **Created**: 2026-03-18 | **Priority**:
- evict_item, get_from_position per policy

### AssociativeCache tests
- **Created**: 2026-03-18 | **Priority**:
- Multi-bucket behavior, put/get flows

### DashSegment tests
- **Created**: 2026-03-18 | **Priority**:
- Stash promotion, probing, eviction cascading

### Edge case tests
- **Created**: 2026-03-18 | **Priority**:
- Empty/full cache, duplicate keys, eviction order

### Integration tests
- **Created**: 2026-03-18 | **Priority**:
- `tests/integration_test.rs` is empty

## Small TODOs

### Pre-allocate bucket storage
- **Created**: 2026-03-18 | **Priority**:
- `Vec::with_capacity` for pre-allocated bucket storage (`dash.rs:32`)

### Derive Copy on EvictionPolicy
- **Created**: 2026-03-18 | **Priority**:
- Derive `Copy` on `EvictionPolicy` to eliminate `.clone()` calls

### Share skills between agents
- **Created**: 2026-03-23 | **Priority**: 4
- Migrate skills from `.claude/skills/` to a shared `.agents/skills/` directory. Update `SKILL.md` files to use relative project paths (e.g., `./caffeine` instead of absolute Windows paths) and ensure compatibility across agents. Create a symlink for Claude's expected location.
