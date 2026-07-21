# Variant v002 Insight

## Helped

- **hit-rate**: The frequency gate at `lfu_counter >= 3` still identifies enough genuinely re-accessed non-hottest items to earn a small incremental hit-rate lift over the position-only parent, confirming that frequency signal in the target tier is real at this workload.

## Hurt

- **per-op-time**: Raising the promotion threshold from ≥ 2 to ≥ 3 did not recover the parent's p99 tail latency — p99 widened from 200 ns to 300 ns — indicating that even at the higher threshold the cascade evictions triggered by frequency-promotions of non-hottest items are frequent enough to produce occasional latency spikes, though p50 remained unaffected.

## Takeaway

The ≥ 3 threshold is a better calibration than ≥ 2 for hit-rate (it keeps the gain without the deeper regression seen in the prior cycle), but it does not restore the parent's tail latency: p99 still inflates because frequency-gated inter-tier promotions displace probing residents and push cascade chains into stash, hitting worst-case eviction paths on some operations. Either a higher threshold (≥ 4 or more) or a structural change that reduces cascade depth is needed before frequency-based promotion can improve both dimensions simultaneously.
