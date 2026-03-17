# Backlog

## Bugs

### Bug 1: TimestampLRU evicts MRU instead of LRU

**File**: `src/shared/traits/bucket.rs:121-127`

```rust
.min_by_key(|(_, item)| item.timestamp.elapsed())
```

`min_by_key(elapsed())` finds smallest elapsed time = most recently accessed item. LRU should evict the **least** recently used (largest elapsed time).

**Fix**: Use `max_by_key(|(_, item)| item.timestamp.elapsed())` or equivalently `min_by_key(|(_, item)| item.timestamp)`.

---

### Bug 2: AssociativeCache ignores num_of_buckets

**File**: `src/associative_cache.rs:28-31`

```rust
let mut buckets = Vec::new();
buckets.push(AssociativeCacheBucket::new(settings.clone()));
```

Always creates exactly 1 bucket regardless of `settings.num_of_buckets`. Missing a loop.

**Fix**: Loop `for _ in 0..settings.num_of_buckets { buckets.push(...) }`.

---

### Bug 3: Swap get_from_position returns wrong item

**File**: `src/shared/traits/bucket.rs:96-101`

```rust
EvictionPolicy::Swap => {
    if position > 0 {
        self.get_items_mut().swap(position, position - 1);
    }
    &self.get_items()[position]
}
```

After `swap(position, position - 1)`, the original item is now at `position - 1`. But we return `items[position]` which is now the neighbor.

**Fix**: Return `&self.get_items()[position - 1]` when `position > 0`, else `&self.get_items()[position]`.

---

### Bug 4: put on existing key doesn't update value

**File**: `src/shared/traits/bucket.rs:37-41`

```rust
if let Some(position) = self.get_position(&item.key) {
    let pushed_item = self.get_from_position(position);
    (pushed_item, None)
}
```

When a key already exists, `put` only calls `get_from_position` (which updates eviction metadata) but never overwrites the stored value with the new one.

**Fix**: Before calling `get_from_position`, update the value: `self.get_items_mut()[position].value = item.value;`

---

### ~~Bug 5: Stale doc example in Item~~ (FIXED)

**File**: `src/shared/item.rs:6`

Doc comment referenced `use dash::data::Item`. Fixed to correct path with `ignore` attribute since `shared` is private.

---

### ~~Bug 6: test_get_index hardcodes hash output~~ (FIXED)

**File**: `src/shared/utils.rs:30-37`

Test asserted `get_index(&123, 10) == 3`, hardcoding the hash output. `DefaultHasher` output is not stable across Rust versions.

**Fix**: Assert `index < size` instead of a specific value.

---

## Performance

- Vec->LinkedList/VecDeque for O(1) eviction in ClassicLRU/FIFO/Swap (`bucket.rs:117`)
- `Vec::with_capacity` for pre-allocated bucket storage (`dash.rs:32`)
- Fixed-size arrays instead of Vec for buckets (`dash_bucket.rs:29`)

## Ownership / API

- Pass settings by reference instead of cloning (`dash.rs:35`, `dash_segment.rs:30`)
- Make `eviction_policy` a reference with lifetime instead of cloned value (`dash_bucket.rs:18`)
- Derive `Copy` on `EvictionPolicy` to eliminate `.clone()` calls
- `put` should return error/signal when key already exists (`bucket.rs:36`)

## Architecture

- Create `Segment` trait for segment abstraction (`dash_segment.rs:21`)
- Blanket `Display` impl for `Bucket` trait (orphan rule issue) (`bucket.rs:153`)
- Re-export `Item` publicly from `lib.rs` (currently inaccessible via `shared`)

## Testing

- Bucket trait tests: evict_item, get_from_position per policy
- AssociativeCache: multi-bucket behavior, put/get flows
- DashSegment: stash promotion, probing, eviction cascading
- Edge cases: empty/full cache, duplicate keys, eviction order
- Integration tests (`tests/integration_test.rs` is empty)
