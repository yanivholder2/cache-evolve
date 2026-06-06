# Variant v002 Implementation Summary

## Hypothesis
Compose v002's frequency-aware eviction policy with v003's proven 75/25 geometry and adaptive divisor formula to test whether implementation efficiency (in-place frequency counting) can be combined with geometry-parameter co-optimization for improvements across both hit-rate and latency.

## Changes Implemented

### 1. Geometry (75/25 Allocation)
**File**: `src/shared/geometry.rs`
- Adopted v003's geometry changes
- Normal buckets: 75% of total buckets (computed as `total_buckets * 3 / 4`)
- Stash buckets: 25% of total buckets
- This provides more stash headroom for evicted items while maintaining the tier structure

### 2. Promotion Threshold (Adaptive Divisor)
**Files**: `src/dash/segments/mod.rs`, `src/dash/segments/dash_segment.rs`
- Adopted v003's promotion threshold formula
- Formula: `max(1, stash_buckets / 4)` (divisor of 4 instead of 3)
- More restrictive than v002's baseline, controls which items get promoted to probing tier
- Prevents excessive tier migration that triggers cascading evictions

### 3. Eviction Policy - FrequencyAwareLRU
**File**: `src/shared/eviction_policy.rs`
- Added new `FrequencyAwareLRU` eviction policy struct
- Updated `EvictionPolicyKind` enum to include `FrequencyAwareLRU` (index 6)
- Updated all associated functions (`name()`, `index()`, `from_usize()`, `from_name()`, `build()`)

**Behavior**:
- `get_from_position()`: 
  - Increments `lfu_counter` on every hit (frequency tracking)
  - Updates `timestamp` to `Instant::now()` on every hit (recency tracking)
  - Returns the same position (in-place modification, no movement)

- `eviction_candidate_index()`:
  - Selects item with minimum composite key: (timestamp, lfu_counter)
  - Prefers items that are **both old AND infrequently accessed**
  - First priority: oldest timestamp (least recently used)
  - Tie-breaker: lowest frequency counter

- `promotion_candidate_index()`:
  - Selects item with maximum composite key: (lfu_counter, recency)
  - Prefers items that are **frequently accessed AND recently visited**
  - First priority: highest frequency counter
  - Tie-breaker: most recent timestamp

### 4. Default Settings
**File**: `src/dash/dash_settings.rs`
- Changed default `eviction_policy` from `ClassicLRU` to `FrequencyAwareLRU`
- Changed default `debug_mode` from `1` to `0` (debug logging disabled)
- Maintains all other defaults unchanged

## Design Rationale

The composition of these changes addresses the cycle 0009 insight:
- **Geometry (v003)**: The 75/25 allocation with divisor-4 threshold proved effective, achieving best-in-class hit-rate (0.7169) while improving latency
- **Eviction Policy (v002)**: FrequencyAwareLRU provides implementation-efficient frequency tracking through in-place counter increments

The key insight is that these concerns are orthogonal:
- **Geometry + Promotion Threshold**: Controls *which items* move between tiers and how stringent that movement is
- **Eviction Policy**: Controls *which items* leave the cache entirely when capacity is exhausted

By combining v003's proven geometric tuning with v002's frequency-aware candidate selection, we test whether:
1. Frequency awareness provides decision value when the geometry is properly balanced
2. The divisor-4 threshold gates unnecessary promotions, making frequency information more meaningful
3. Joint optimization of both axes improves performance beyond either alone

## Test Expectations

The variant should:
- **Pass build gate**: Correctly compiles with new FrequencyAwareLRU policy
- **Pass contract gate**: Basic Cache trait operations work correctly
- **Pass universal invariant gate**: Hit/miss behavior and phantom items remain valid
- **Pass invariant gate**: Slot legality, capacity bounds, and structural properties maintained

Performance expectations:
- Hit-rate: Should maintain or exceed v003's 0.7169 (frequency awareness + proper geometry)
- Latency: Should improve or maintain v003's performance (no movement overhead in get_from_position)
- Metadata: Item size unchanged (still 40 bytes for i64,i64)
