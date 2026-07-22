# Variant v003 Insight

## Helped

- **hit-rate**: Gating promotion distance on `lfu_counter` — 3 steps for repeat accesses, 1 step for first access — fast-tracks confirmed hot keys to the head before eviction pressure can reach them on the Zipfian workload, where the majority of gets in steady state land on already-seen keys.

## Hurt

- **per-op-time**: The `get` path now writes `lfu_counter` and performs up to 3 swaps per access (vs. v002's single write-free swap), and put p99 doubled; the additional memory writes on the get path appear to increase cache-line pressure enough to surface in tail latency, even though the p50 summary remained pinned at the measurement floor.

## Takeaway

Frequency-gated multi-step promotion yields a real hit-rate gain over v002's unconditional single-step policy: the counter-based branch effectively distinguishes the hot-key population and rushes them to head position in fewer accesses. However, the cost is a write to `lfu_counter` on every get plus up to two extra swaps, which raises put p99 — suggesting that on this workload the tradeoff of more memory writes per get for marginally better ordering is borderline, and any further increase in per-access work will need to be weighed against latency tail effects.
