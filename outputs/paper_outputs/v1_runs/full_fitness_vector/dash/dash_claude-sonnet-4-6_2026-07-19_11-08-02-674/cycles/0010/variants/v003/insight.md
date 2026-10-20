# Variant v003 Insight

## Helped

_(no dimensions improved)_

## Hurt

- **hit-rate**: Lowering the probing re-entry threshold to ≥1 causes items to be recycled back to target on their very first probing hit — before they can accumulate tenure in the probing tier — increasing per-item churn between tiers and degrading steady-state hit rate on the Zipfian trace relative to the ≥2 threshold in the parent.
- **metadata-size**: v003's overlay contains only `dash_segment.rs`, so the parent's custom `item.rs` (which reduced `Item<u64,u64>` from 40 to 24 bytes) was not re-applied in this variant's evaluation, causing the item size to revert to its baseline.
- **per-op-time**: The get p99 tail latency increased while p50 held steady, consistent with the more aggressive recycling path now firing on nearly every probing hit when target has space, adding move-overhead on high-frequency access patterns.

## Takeaway

The hypothesis that every probing hit should immediately qualify for re-entry was falsified: dropping the threshold from ≥2 to ≥1 increases tier churn faster than it shortens the promotion ladder, and on this Zipfian trace the vacancy window is wide enough that the gate was already firing frequently at ≥2 — the extra sensitivity at ≥1 provides no additional lift and instead destabilizes hot items by bouncing them out of probing before they have a chance to re-earn their slot through the normal MRU path. The parent's rationale that the vacancy condition (not the counter) is the binding constraint appears not to hold: the counter threshold itself does meaningful filtering work.
