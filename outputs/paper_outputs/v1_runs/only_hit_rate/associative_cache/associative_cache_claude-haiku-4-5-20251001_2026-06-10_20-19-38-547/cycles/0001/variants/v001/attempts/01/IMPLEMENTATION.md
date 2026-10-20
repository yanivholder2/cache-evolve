# Stash Buffer Implementation for Associative Cache

## Hypothesis
Add a bounded overflow stash buffer to capture evicted items and provide a second-chance lookup on cache misses, raising hit rate by reducing loss of evicted data.

## Implementation Summary

### Overview
Modified `AssociativeCache<K, V>` to include a secondary buffer (stash) that captures items evicted from their primary buckets. When a cache miss occurs, the stash is checked for the item before returning None, allowing recovery of items that would otherwise be permanently lost.

### Changes Made

#### 1. Data Structure Extension (`src/associative_cache/mod.rs`)
- **New fields in `AssociativeCache<K, V>`:**
  - `stash: Vec<Item<K, V>>` - bounded vector storing evicted items
  - `stash_capacity: usize` - maximum capacity of the stash

- **Capacity calculation:** Approximately 10% of total cache capacity (with minimum of 1)
  ```rust
  let total_capacity = settings.num_of_buckets * settings.bucket_size;
  let stash_capacity = std::cmp::max(1, total_capacity / 10);
  ```

#### 2. put() Method Enhancement
- **Added stash cleanup:** Before inserting a key, any stale version in the stash is removed
  - Prevents duplicate keys when the same key is inserted after being evicted
- **Eviction capture:** When a bucket evicts an item, it's now added to the stash
  ```rust
  if let Some(evicted_item) = evicted {
      self.add_to_stash(evicted_item);
  }
  ```

#### 3. Stash Management (add_to_stash Helper)
- **FIFO eviction policy:** When stash reaches capacity:
  - Remove the oldest item (index 0)
  - Add the new item to the end (push)
  - Maintains a sliding window of recent evictions

#### 4. get_and_update_item() Method Enhancement
- **Primary lookup:** Check bucket first (unchanged behavior)
- **Secondary lookup:** On bucket miss, search the stash
- **Re-insertion logic:** On stash hit:
  1. If the primary bucket has space: re-insert the item and remove from stash
  2. If the primary bucket is full: return the value from the stash copy
- **Return behavior:**
  - Returns reference to value in bucket if re-inserted
  - Returns reference to value in stash if re-insertion not possible
  - Returns None if not found anywhere

### Key Design Decisions

1. **Bounded Stash**: Uses 10% capacity to balance memory overhead and recovery benefit
2. **FIFO Stash Eviction**: Simple and deterministic, complements the existing policies
3. **Lazy Re-insertion**: Items are re-inserted when accessed and space is available, rather than proactively
4. **Stale Cleanup**: Removes old stash copies before inserting new versions to prevent duplicates

### Behavioral Impact

- **Hit Rate**: Expected to improve when workloads have temporal locality and re-access evicted items
- **Memory Overhead**: Fixed additional memory for stash (10% of cache capacity)
- **Performance**: Minimal cost - secondary lookup only on primary bucket miss
- **Correctness**: Maintains cache invariants (never has duplicate keys in bucket)

### Test Coverage

Added 6 unit tests validating:
1. Evicted items are captured in stash
2. Stashed items can be recovered via secondary lookup
3. Re-inserted items are removed from stash
4. Stash FIFO eviction when at capacity
5. Backward compatibility with existing tests

## Compatibility

- ✅ Preserves `Cache<K, V>` trait interface
- ✅ Compatible with all existing eviction policies
- ✅ No changes to gate-integrity files
- ✅ Full-autonomy scope (no extension-points modifications)
