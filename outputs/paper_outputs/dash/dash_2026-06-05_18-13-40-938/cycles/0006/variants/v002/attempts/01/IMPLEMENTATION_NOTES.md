# Variant v002 Implementation Notes

## Hypothesis (H02)
Skew normal bucket allocation toward probing tier (18 probing, 10 target) instead of equal distribution (14 target, 14 probing) to provide more capacity for hot-tier residents and reduce eviction cascades, targeting hit rate improvement.

## Implementation

### Changes Made

#### 1. Modified `src/dash/segments/mod.rs` (SegmentCore)

**Added fields to SegmentCore struct:**
- `num_target_buckets: usize` - Number of buckets in target tier
- `num_probing_buckets: usize` - Number of buckets in probing tier

**Updated `try_new()` method:**
- Compute skewed bucket distribution using the formula: `(num_of_normal_buckets * 10) / 28`
- This maintains the original 10:18 = 35.7%:64.3% split proportionally across all bucket sizes
- For 28 buckets: `num_target = 10`, `num_probing = 18` ✓
- For 4 buckets (test): `num_target = 1`, `num_probing = 3`

**Added instance methods:**
- `target_bucket_for_key(&self, key: &K) -> usize` - Maps key to target bucket [0, num_target_buckets)
- `probing_bucket_for_key(&self, key: &K) -> usize` - Maps key to probing bucket [num_target_buckets, segment_size)

**Updated methods to use new bucket mapping:**
- `contains_key()` - Checks both target and probing buckets correctly
- `check_slot_legality()` - Validates items are in legal buckets per the skewed split
- `put_at()` - Routes items to correct buckets for Target and Probing slots

#### 2. Modified `src/dash/segments/dash_segment.rs` (DashSegment)

**Updated promotion logic:**
- `promote_to_target()` - Uses `core.target_bucket_for_key()`
- `promote_to_probing()` - Uses `core.probing_bucket_for_key()`
- `demote_from_normal_bucket()` - Uses `core.target_bucket_for_key()` for correct demotion
- `promotion_get()` - Uses both methods to locate items correctly

**Updated all tests:**
- Dynamically compute bucket indices and sizes based on segment configuration
- Tests now work with any bucket configuration (default 28, test settings 4, etc.)
- Use `seg.core().num_target_buckets` and `seg.core().num_probing_buckets` for bucket size parameters

### Design Rationale

**Dynamic Split Computation:**
The implementation uses a proportional formula rather than hardcoding 10:18. This ensures:
1. The hypothesis is satisfied for the baseline 28-bucket configuration
2. Tests work correctly with any bucket size
3. The same ~35.7%:64.3% split is maintained across all configurations

**Bucket Index Mapping:**
- Target buckets: `hash(key) % num_target_buckets` → range [0, num_target_buckets)
- Probing buckets: `num_target_buckets + (hash(key) % num_probing_buckets)` → range [num_target_buckets, segment_size)

This ensures:
- Each key has exactly one target bucket
- Each key has exactly one probing bucket
- Buckets are contiguous: target tier occupies [0, num_target_buckets), probing tier occupies [num_target_buckets, segment_size)
- The split can scale to any total bucket count

### Verification

For the default settings (28 buckets):
- `num_target = (28 * 10) / 28 = 10` ✓
- `num_probing = 28 - 10 = 18` ✓
- Ratio: 10:18 = 35.7%:64.3% ✓
- Total: 10 + 18 = 28 ✓

For test settings (4 buckets):
- `num_target = (4 * 10) / 28 = 1`
- `num_probing = 4 - 1 = 3`
- Ratio: 1:3 = 25%:75% (maintains proportional skew)
- Total: 1 + 3 = 4 ✓

All tests updated to use dynamic computation and should pass with any valid bucket configuration.

### Contract Compliance

The overlay maintains the `Cache<K, V>` trait contract and SegmentCore invariants:
- Slot legality: Every item is in either target or probing bucket, correctly derived from its key
- Capacity conservation: Total buckets remain constant
- Promotion/demotion logic: Unchanged, uses new bucket mapping
- Eviction discipline: Preserved through correct bucket management
