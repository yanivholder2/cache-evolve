# Variant v002 Insight

## Helped

- (none) — the targeted per-op-time summary (get-p50) did not move: replacing the unconditional `slot.1 = (..).min(MAX_FREQ)` store with an `if slot.1 < MAX_FREQ` branch left the median get exactly where it was, showing the freq store was never the bottleneck the median paid for.

## Hurt

- per-op-time: get-p99 regressed sharply while p50 held flat — the new data-dependent conditional on `slot.1 < MAX_FREQ` is mispredictable across hits, so the saturated-vs-unsaturated mix that the old branchless clamp absorbed now shows up as tail-latency jitter; put p50/p99 also drifted slightly up.

## Takeaway

Eliding the saturated-key freq write did not lower get-p50 because that median was already at the measurement floor — the freq store under `get_mut` was not the cost the median get paid. Trading a branchless clamped store for a value-dependent branch bought no median win and cost tail latency to misprediction, so this is the wrong lever for the per-op-time axis on this workload.
