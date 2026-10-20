# Variant v003 Insight

## Hurt

- **hit-rate**: Periodic decay did not improve frequency estimation accuracy; the 0.7409→0.7405 drop suggests decay-induced counter resets eliminate useful history without measurably reducing eviction churn on this workload.
- **per-op-time**: Calling `decay_if_needed()` at the start of both `increment()` and `estimate()` added a conditional check on every operation; the p99 latency spike for get operations (154→243 ns) indicates the decay operation itself (bitwise right-shift on all counters) is expensive and triggers often enough to degrade tail latency.

## Helped

(None)

## Takeaway

Periodic decay on high-frequency sketches trades latency for unproven hit-rate gains. Although the 500k-operation interval was tuned in a prior cycle, applying it to the width-8 geometry proved harmful: the decay mechanism's O(depth×width) cost on every operation outweighs the minimal hit-rate benefit, and the latency penalty contradicts the hypothesis that fresher frequency signals would reduce eviction churn. The 8-byte metadata cost (op_count field) is negligible compared to the execution overhead—suggesting that periodic reset approaches may be fundamentally misaligned with admission decisions driven by high-collision, short-lived sketches.
