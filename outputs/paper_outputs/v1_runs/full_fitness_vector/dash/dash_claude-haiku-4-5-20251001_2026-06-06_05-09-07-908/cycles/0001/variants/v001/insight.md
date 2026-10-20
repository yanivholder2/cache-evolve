# Variant v001 Insight

## Helped

- **hit-rate**: Back-insert keeps promoted items at MRU position longer, reducing unnecessary demotion cascades and keeping hot items resident on Zipfian traces.
- **per-op-time**: Eliminating the forced LRU climb after promotion reduces latency for both get (p50 -43%, p99 -50%) and put operations, as items avoid demotion-repromote cycles.

## Hurt

(none)

## Takeaway

Back-insert promotion succeeds in its design goal: promoted items no longer waste access budget re-earning MRU position. By landing at the back instead of the front of their destination bucket, frequently accessed items stay in hot tiers longer, reducing both demotion pressure and access latency. This is a clean win for Zipfian workloads where a small set of hot keys dominate traffic.
