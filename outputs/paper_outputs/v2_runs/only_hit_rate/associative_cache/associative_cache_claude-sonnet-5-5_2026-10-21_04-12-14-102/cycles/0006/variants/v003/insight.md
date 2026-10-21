# Variant v003 Insight

## Helped

- hit-rate: Doubling ways per bucket (with half the buckets, same total slots) lets more keys share a set. The positional 4-slot probation/protected split then keeps one-touch keys from displacing re-referenced ones. Together these cut conflict and scan-induced misses on the LIRS workload, and the hit-rate rose clearly over baseline.

## Hurt

- metadata-size: The fitness vector has no entry for this dimension, so the effect is unmeasured. The design adds no counters, ghost state or stamps, so no growth is expected.
- per-op-time: Also not reported. Wider buckets mean longer scans and the segment moves add work per hit, so latency is a plausible cost. This variant gives no evidence either way.

## Takeaway

Wider associativity and a fixed-size slot-based probation segment do stack on the baseline, and they need no extra metadata. The evaluation only recorded hit-rate, so the latency cost of longer bucket scans and promotion and demotion shuffles is still untested. Any follow-up should measure per-op-time before treating this as a clear win.
