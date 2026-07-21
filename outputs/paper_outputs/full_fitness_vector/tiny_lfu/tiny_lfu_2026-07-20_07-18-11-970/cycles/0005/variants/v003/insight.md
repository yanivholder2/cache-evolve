# Variant v003 Insight

## Helped

- `hit-rate`: The stricter `> victim_freq.saturating_add(1)` gate blocked low-delta incoming keys that the parent's plain `>` comparison would have admitted, yielding a small but additive hit-rate gain on top of v002's already-improved baseline — confirming the two mechanisms contribute independently.

## Hurt

- `per-op-time`: Both get and put p50 latency doubled relative to v002, matching the pattern of the prior cycle's isolated stricter-gate variant; the regression survived onto v002's base unchanged, which shows the overhead originates in the altered node-array residency pattern (different keys occupying slots) rather than in eviction-path execution time — an effect the 1-in-8 amortisation cannot address because it only reduces probe frequency, not the memory-access pattern of the nodes that remain.

## Takeaway

Stacking the stricter admission gate on v002's amortised best-of-2 base confirmed that the two hit-rate signals accumulate but that their latency effects are independent: the gate's doubling of p50 comes from changing *which* nodes are resident (altering CPU cache-line access patterns for both get and put), not from the cost of the gate comparison itself, so v002's eviction amortisation — which targets probe overhead — cannot neutralise it. Further exploration of the stricter gate should either tolerate the latency cost or seek a softer threshold (e.g., `> victim_freq + 0` on the 1-in-8 probe path only) that limits residency disruption while preserving admission selectivity.
