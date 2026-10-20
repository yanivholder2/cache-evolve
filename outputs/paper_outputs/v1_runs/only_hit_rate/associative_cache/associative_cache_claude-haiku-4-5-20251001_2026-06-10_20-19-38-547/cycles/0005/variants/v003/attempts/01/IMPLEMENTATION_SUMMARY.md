# Variant v003: Collision-Aware Selective Stash Insertion

## Summary

This variant implements collision-aware selective stash insertion to preserve only evicted items from high-contention buckets. By tracking collisions per bucket and filtering stash insertion based on collision count (≥2), the cache reduces stash pollution from low-value evictions while leveraging v001's geometric collision reduction.

## Changes

### 1. `src/associative_cache/associative_cache_bucket.rs`
- **Added collision tracking**: New `collision_count: usize` field to `AssociativeCacheBucket`
- **New methods**:
  - `increment_collision_count()`: Increments counter when an eviction occurs
  - `reset_collision_count()`: Resets counter for periodic workload pattern adaptation

### 2. `src/associative_cache/mod.rs`
- **Enhanced cache structure**:
  - Added `operation_count: usize` to track total operations
  - Added `reset_period: usize` (default: 1000) for periodic counter resets
  
- **Modified `put()` method**:
  - Captures evicted items only if source bucket's `collision_count >= 2`
  - Items from buckets with 0-1 collisions are discarded (filtered)
  - Collision counter incremented on each eviction
  - Operation counter incremented after each put
  - Collision counters reset every `reset_period` operations
  
- **Added `reset_collision_counters()` method**:
  - Resets all bucket collision counters to 0
  - Called periodically to reflect current workload patterns

### 3. `src/shared/geometry.rs`
- **Inherited from v001**: Geometric collision reduction
  - Reduces bucket count by ~71% (67-75% range)
  - Increases bucket size proportionally to maintain total capacity
  - Spreads items across larger buckets to reduce collision frequency

## Design Rationale

### Collision-Based Filtering
- **Problem**: v001's stash mechanism captures ALL evictions, which can fill the stash with items that were rarely accessed anyway
- **Solution**: Only preserve evictions from buckets experiencing persistent contention (collision_count ≥ 2)
- **Benefit**: Items from lightly-loaded buckets (0-1 collisions) are natural noise; items from hot buckets are more likely to be re-accessed

### Periodic Reset
- **Design**: Reset collision counters every 1000 operations
- **Rationale**: Allows cache to adapt to changing workload patterns
- **Behavior**: If a bucket becomes cold after being hot, counter resets and reflects new behavior

### Synergy with v001's Geometry
- **Geometry**: Fewer, wider buckets reduce baseline collision frequency
- **Filtering**: Once baseline collisions are reduced, selective stash insertion captures high-value recoveries
- **Result**: Improved hit rate by preserving stash capacity for highest-value re-uses

## Implementation Details

### Stash Insertion Decision Tree
```
if evicted_item:
    increment bucket.collision_count
    if bucket.collision_count >= 2:
        add_to_stash(evicted_item)
    else:
        discard evicted_item
```

### Reset Behavior
- Resets triggered at operation counts: 1000, 2000, 3000, etc.
- All buckets reset simultaneously
- Allows cache to detect workload phase changes

### Preserved from v001
- FIFO stash eviction policy
- 25% stash capacity (max(1, total_capacity / 4))
- Secondary lookup in stash on primary bucket miss
- Stash re-insertion when space available

## Testing

Added comprehensive tests to verify:
1. Collision counter tracking per bucket
2. Stash insertion only occurs after 2+ collisions
3. Items from 0-1 collision buckets are discarded
4. Stash FIFO eviction continues to work
5. Periodic collision counter resets
6. Secondary lookup and re-insertion from stash still functions

## Expected Impact

**Improvements**:
- Reduced stash churn by filtering low-value evictions
- Better stash capacity utilization for high-value items
- Synergistic effect with v001's geometry optimization

**Trade-offs**:
- First eviction from any bucket is always lost (collision_count=1)
- Items from buckets that suddenly become hot after a reset may be lost until collision_count reaches 2
- Requires operation counting (minor overhead)
