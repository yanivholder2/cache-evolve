# Cross-Cycle Insights

## Recurring wins

- **Tight admission filtering reliably improves latency**: Both cycles consistently show that stricter promotion thresholds and filtering gates reduce main-queue contention and deliver 28–66% per-operation latency gains. This is the most robust lever identified so far.
- **Reduced resident-set contention benefits workload performance**: Smaller active queues translate not just to lower lookup chains but also to reduced eviction overhead, suggesting the benefit extends beyond direct latency to cache locality and GC effects.
- **Operation-specific optimization is achievable**: Variants successfully isolated latency improvements in specific operations (median puts, gets) by tuning admission and eviction timing, showing granular control is possible.

## Recurring failures

- **Hit-rate recovery remains fundamentally decoupled from latency gains**: Across both cycles, every attempt to recover hit-rate—via queue expansion (v002), eviction deferral (v003), or threshold adjustment (v001)—failed to meaningfully offset the cost of tight admission. The strict promotion threshold is the binding constraint, not downstream mechanisms.
- **Simple queue-sizing and eviction-timing tweaks address the wrong bottleneck**: v002's expansion and v003's lazy eviction both discovered that tuning capacity or eviction frequency doesn't overcome the admission filter's gate. The bottleneck is upstream (who gets admitted), not downstream (how they're evicted).
- **No variant has achieved latency *and* hit-rate parity simultaneously**: Each winner optimizes a subset of metrics—median put, tail latency, get efficiency—at the expense of others, indicating the current design space is genuinely constrained, not poorly explored.

## Open questions

- Can admission mechanisms beyond binary frequency thresholds (decay-based ranking, recency weighting, bimodal buckets) recover hit-rate without abandoning the latency gains?
- Is there a compound parameter variation (e.g., freq > 2 with 30% queue + 85% eviction threshold) that balances get and put latency across both median and tail, or is the trade-off unavoidable?
- Do these tight-filtering strategies degrade on workloads with broader, less-Zipfian access patterns, or is the latency benefit universal?
- Can ghost tracking or deferred-promotion heuristics preserve warm-but-not-hot items while keeping the admission filter tight?

## Per-policy notes

- **Frequency-threshold promotion (v001 variants)**: Raising the threshold (freq > 2 to freq > 3) tightens admission and improves latency, but hits-rate decline is proportional to strictness. Threshold tuning alone is insufficient to recover hit-rate; the cost is inherent to the strategy.
- **Ghost-hit and queue-expansion hybrids (v002)**: Expanding the small queue or re-routing ghost hits doesn't help if the admission threshold blocks items before they accumulate sufficient frequency. Larger windows compound eviction churn, making the problem worse.
- **Lazy-eviction and deferred-timeout approaches (v003)**: Deferring main-queue eviction improves put efficiency by reducing eviction-call frequency, but doesn't address admission-filter constraints on hit-rate. The speedup is real but narrow—get latency degrades, and hit-rate remains penalized.
