# Variant v002 Implementation: H02 - Stash Capacity at 30%

## Summary

This variant implements a stash mechanism for the associative cache with 30% capacity (up from 25% in previous cycles). The stash serves as an overflow buffer for items evicted from their primary buckets, with FIFO eviction policy.

## Changes Made

Modified `src/associative_cache/mod.rs`:

### 1. Data Structure
- Added `stash: Vec<Item<K, V>>` field to `AssociativeCache` struct
- Added `stash_capacity: usize` field to track maximum stash size
- Stash capacity is calculated as: `(total_capacity * 3) / 10` (30% of bucket capacity)

### 2. Initialization
- In `try_new()`: Initialize empty stash with proper capacity calculation
- Stash is pre-allocated with `Vec::with_capacity()` for efficiency

### 3. Put Operation
- When a bucket evicts an item (due to being full), the evicted item is added to the stash
- Uses `put_in_stash()` helper method with FIFO eviction:
  - If stash is at capacity, remove oldest item (index 0)
  - Push new item to end (newest)

### 4. Get Operation
- Enhanced `get_and_update_item()` with two-level lookup:
  1. First, try to find key in primary bucket (hash-based lookup)
  2. If miss, search stash linearly
  3. If found in stash, remove from stash and re-insert into primary bucket
  4. Handle bucket evictions during re-insertion (evicted items go back to stash)

## Implementation Details

**FIFO Eviction Strategy**: 
- Oldest items (lower indices) are evicted first
- New items are appended to the end
- `self.stash.remove(0)` removes oldest, `self.stash.push(item)` adds newest

**Locality Preservation**:
- Items stay in primary buckets when possible (no stash lookup overhead)
- Stashed items are only retrieved on explicit get misses
- Re-insertion attempts to restore items to their hash buckets

**Invariant Compliance**:
- Slot legality: Primary bucket items are always in their hash bucket
- Stash items don't violate slot legality (they're overflow)
- Capacity conservation: Stash uses budget independent of bucket layout
- No phantom items: All resident items (in buckets or stash) are retrievable

## Testing

Updated test `eviction_when_bucket_full()` to verify:
- Items evicted to stash are still retrievable
- Stash recovery restores items to primary buckets
- Multiple accesses maintain cache consistency
