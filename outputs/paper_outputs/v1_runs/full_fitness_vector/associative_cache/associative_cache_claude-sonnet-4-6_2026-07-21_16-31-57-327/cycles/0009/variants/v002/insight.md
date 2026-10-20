# Variant v002 Insight

## Helped

_(none)_

## Hurt

- **hit-rate**: Explicitly targeting one-hit-wonder items (`lfu_counter == 1`) in the new second eviction pass made them more eviction-vulnerable than in the parent, where all accessed items (`counter ≥ 1`) had equal second-chance protection; on this Zipfian workload many items accessed exactly once were re-requested soon after, so preferentially evicting them reduced hits rather than filtering true garbage.
- **metadata-size**: The v002 overlay only modifies `eviction_policy.rs` and does not carry forward the 32-byte compact `Item` layout introduced by the parent's `item.rs`; without that file in the overlay, the build falls back to the base codebase's 40-byte struct, silently undoing v003's size reduction.
- **per-op-time**: The second eviction scan pass — up to 8 additional comparisons executed whenever no generation-0 candidate is found in the cold window — raised the put p99 tail latency, though the p50 and summary metric were unaffected.

## Takeaway

The two-pass eviction design assumes that items accessed exactly once are lower-value than confirmed-warm items and can be safely prioritised for eviction, but the measured hit-rate regression shows this assumption does not hold on this workload: single-access items in the cold window are often accessed again and should retain the same second-chance protection as multi-access items. Separately, and unrelated to the eviction logic, omitting `item.rs` from the overlay silently reverted the struct compaction the parent had achieved — any future variant building on this two-generation counter must explicitly include the compact `Item` layout to avoid regressing metadata-size.
