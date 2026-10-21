# Cycle 0010 Insight

## Cohort outcome

Three variants were tried: a wider 3-bit reuse counter with the all-hot drop rule (v001), removal of the all-hot-bucket drop rule with a slot-position tie-break (v002), and an adaptive second-sighting admission gate with a reuse flag (v003). v002 was labelled Winner, but no variant showed a clear hit-rate improvement. The Winner label is probably selection noise again.

## Patterns

- This is the fifth cycle in a row with noise-level outcomes. Every change either regressed slightly or sat at the noise floor.
- More counter resolution (v001) hurt. Slow-decaying, finer state over-protects stale entries, so recency-responsive aging matters more than separating frequent from occasional reuse.
- The drop rule seems to give a small amount of real protection, since removing it (v002) cost a little. Its worth is marginal, and the simpler path's cost savings were never measured.
- Adaptive admission (v003) added moving parts (window counters, mode switch, unaged bit filter) and still lost to the plain baseline. Its trigger signal is laggy and noisy.
- The measurement gap persists. Only hit-rate on one trace and one size is reported. Per-op-time and metadata-size are missing, so simplification and complexity trade-offs cannot be judged.

## Open questions

- Is any hit-rate headroom left on workload-a, or is every lever at the noise floor?
- Does the simpler v002 path win on per-op-time once that is measured?
- Do other workloads and cache sizes show effects that this trace hides?

## Continuation

Stop tuning counter width, drop rules, and admission gating on workload-a. Treat the incumbent as unchanged, and treat v002 as a simplification candidate only if cost data supports it. Next cycle should first add per-op-time and metadata-size to the fitness vector and validate on other workloads and cache sizes. After that, explore levers outside admission, placement and eviction state, such as adaptive sizing or scan-resistance for other access patterns.
