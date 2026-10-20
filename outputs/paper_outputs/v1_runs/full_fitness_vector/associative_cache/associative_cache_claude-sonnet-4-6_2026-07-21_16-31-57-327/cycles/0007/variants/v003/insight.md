# Variant v003 Insight

## Helped

## Hurt

- **hit-rate**: Gating the write-once flag on reaching position 0 (rather than on first access) did not free meaningful eviction capacity — items accessed only once or twice rarely occupied protected slots under the existing second-chance scan, so tightening the threshold produced no measurable gain on this Zipfian workload.
- **metadata-size**: The current overlay only changed `eviction_policy.rs` and did not re-include the 32-byte `item.rs` from the parent; `item.rs` reverted to the base layout, inflating the per-item struct from 32 to 40 bytes.
- **per-op-time**: The larger item struct (40 vs 32 bytes) increases cache-line pressure on `put` paths that iterate or displace items, pushing put p99 latency up compared to the parent.

## Takeaway

Moving the protection flag from "first touch anywhere" to "first arrival at position 0" is neutral on hit rate for this workload, which suggests the once-accessed mid-range items that the hypothesis targeted were not a meaningful drain on eviction quality under the existing 8-slot second-chance scan. The dominant regression in this variant came from accidentally losing the compact 32-byte item layout: H03 only overlaid `eviction_policy.rs`, so the 32-byte `item.rs` from the parent was not inherited, undoing the metadata-size win and introducing a put-latency penalty.
