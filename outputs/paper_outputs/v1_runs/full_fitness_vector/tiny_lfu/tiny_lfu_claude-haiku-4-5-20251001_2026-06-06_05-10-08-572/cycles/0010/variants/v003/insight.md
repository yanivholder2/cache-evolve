# Variant v003 Insight

## Helped

- per-op-time: Median latency improved (~16% faster on both get and put p50) due to unconditional admission of updates to existing keys, eliminating frequency comparison overhead on the hot path.

## Hurt

- per-op-time: Tail latency (get p99) regressed slightly, suggesting occasional contention or variance in the admission path for hot workloads.

## Takeaway

Asymmetric admission based on entry type—always admitting updates while gating new arrivals—delivers measurable latency gains by streamlining the common case (existing-key updates). However, it does not improve hit-rate by itself: this variant's 0.7362 hit-rate is unchanged from baseline, confirming that earlier work's hit-rate improvements required stricter frequency thresholds, not merely distinguishing between update and insertion semantics.
