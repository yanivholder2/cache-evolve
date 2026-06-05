# Variant v001 Insight

## Helped

- hit-rate: Lowering the small-bucket fallback from `len < 4` to `len < 2` and switching the gate to the freshest half (`item_idx >= len - len.div_ceil(2)`) lets the lever finally engage on len-3 and len-4 target buckets — exactly the sizes that dominate this heavily-hashed trace — so a few more warm keys reach the probing tier and hit-rate edges up.

## Hurt

- hit-rate: The gain is still only a sliver: `len == 2` buckets are deliberately held to hottest-only (threshold `len - 1`) to avoid over-admitting, and many target buckets are single-resident (`len < 2`) where the gate degenerates to the baseline regardless, so the widened branch reaches only a thin slice of accesses.

## Takeaway

Replacing the quarter gate's `len < 4` cliff with a fractional `div_ceil(2)` half-gate did unlock promotion on the len-3/len-4 buckets the parent's insight flagged as starved, confirming the bottleneck was bucket occupancy, not gate direction. But the unlock is marginal because the trace's target buckets skew toward len-1 and len-2, where the gate is intentionally or unavoidably hottest-only — so widening gate fraction has nearly exhausted its headroom; further hit-rate must come from making target buckets denser, not from loosening the rank threshold.
