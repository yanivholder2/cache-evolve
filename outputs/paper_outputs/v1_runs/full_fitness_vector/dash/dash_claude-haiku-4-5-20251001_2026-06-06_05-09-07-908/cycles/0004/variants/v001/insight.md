# Variant v001 Insight

## Helped

- **hit-rate**: Frequency-gating target→probing promotion on lfu_counter ≥ 2 prevents single-access false-positives from polluting the probing tier, allowing the hot-set to be preserved more faithfully and improving overall hit accuracy on the concentrated 75/25 geometry.

- **per-op-time (put)**: Reduced promotion frequency lowers contention in the probing bucket and amortized cascade-demotion overhead, marginally improving put latency.

## Hurt

- **per-op-time (get)**: The added lfu_counter comparison check (line 170) and intra-promotion counter increments introduce latency on target-hit paths; items also spend more cycles in the target bucket before reaching the promotion threshold, extending the average lookup cost for operations that would have promoted immediately under geometry-only gating.

## Takeaway

Frequency-gated promotion composes beneficially with the 75/25 geometry on hit-rate, confirming that the gate's selectivity scales well to even more concentrated bucket splits. The trade-off is measurable: more accurate hit decisions come at the cost of slightly longer get latency, as items linger in the target tier longer before earning promotion, suggesting a frontier where hit-accuracy and latency diverge under this workload.
