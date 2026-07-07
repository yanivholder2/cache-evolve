# Three-Tier Stash Implementation (v002)

## Overview

This variant implements a three-tier stash system (hot/warm/cold) to reduce hot-tier churn from oscillating items and better match Zipfian re-access distribution in cache workloads.

## Changes Made

### 1. Geometry (`src/shared/geometry.rs`)

**Design**: Identity geometry - returns the harness-provided bucket split unchanged.

**Rationale**: 
- Stash tiers are managed entirely within AssociativeCache
- No geometry modification needed; stash is supplementary to buckets
- Avoids integer division issues that would arise from capacity reduction
- Stash capacity is derived as a fraction of bucket capacity

### 2. AssociativeCache (`src/associative_cache/mod.rs`)

**New Structure**:
- Added three Vec fields: `hot_stash`, `warm_stash`, `cold_stash`
- Added capacity fields: `hot_stash_capacity`, `warm_stash_capacity`, `cold_stash_capacity`

**Capacity Allocation**:
- Hot stash: 10% of bucket capacity
- Warm stash: 10% of bucket capacity  
- Cold stash: 5% of bucket capacity

**Key Methods**:

1. **try_new()**: Initializes stash Vecs with pre-allocated capacity
2. **put()**: Inserts into bucket; evicted items → cold stash
3. **get_and_update_item()**: Implements lookup sequence with promotion
4. **Promotion helpers**:
   - `promote_cold_to_warm()`: Cold hit → warm, evict LRU warm to cold if full
   - `promote_warm_to_hot()`: Warm hit → hot, evict LRU hot to warm if full
   - `insert_into_cold_stash()`: Handles cold insertion and LRU eviction

## Lookup and Promotion Flow

```
GET(key):
  1. Check bucket → return if found
  2. Check hot stash → update timestamp, return
  3. Check warm stash → promote to hot, return
  4. Check cold stash → promote to warm, return
  5. Return None if not found
```

**Promotion Cascade Example**:
- Item in cold, warm is full, hot is full
- Remove from cold, promote to warm
- Find LRU in warm, move to cold
- Insert cold item into cold (might evict cold LRU)
- Remove LRU from hot, push to warm
- Insert new item into hot

## LRU Eviction

Uses `Item::timestamp` field for LRU tracking:
- Updated on every item creation with `Instant::now()`
- Updated on every tier promotion
- Updated on hot tier access (remove/re-push)
- LRU selection: `min_by_key(|item| item.timestamp)`

## Memory Safety

- Pre-allocated Vec capacity prevents reallocation during push/pop
- References returned from `last().unwrap()` are valid within method scope
- No unsafe code required

## Hypothesis Alignment

✓ Three-tier (hot/warm/cold) stash implementation
✓ Items evicted from buckets enter cold tier
✓ Cold→warm→hot promotion on access
✓ LRU eviction within each tier
✓ Lookup sequence: buckets → hot → warm → cold
✓ Timestamp tracking for LRU determination
✓ Reduces hot-tier churn through buffering in warm tier

## Testing Notes

Existing tests should continue to pass:
- Items evicted from buckets are preserved in cold stash
- Multiple accesses follow promotion sequence
- LRU items within tiers are evicted correctly
- put/get round-trips work through all tiers

## Future Optimizations

- Vector-based lookup could be optimized with hash tables for large stashes
- Timestamp could be replaced with access counter for faster comparison
- Separate promotion logic could be extracted into StashTier struct
