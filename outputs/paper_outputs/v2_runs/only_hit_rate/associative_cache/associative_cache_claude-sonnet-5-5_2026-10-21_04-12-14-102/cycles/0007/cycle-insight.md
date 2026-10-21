# Cycle 0007 Insight

## Cohort outcome

Three probes were tried: doubled associativity as an increment on the best lean split with probation scaling at slots/16 (v001), the same doubling with probation held at an absolute 4 slots (v002), and a policy-preserving speed refactor of the rotate-based split (v003). Only v001 was a Winner, with a very small hit-rate gain. v002 lost marginally, and v003 was a hit-rate tie that cannot be judged.

## Patterns

- Wider buckets do stack on the incumbent, but the increment is tiny. The remaining misses look like capacity or recency-ordering misses, not bucket-conflict misses.
- Proportional probation sizing (slots/16) held up better than an absolute 4 slots on wider buckets. Associativity and probation size are only weakly coupled, and the absolute-scan-length reading is not supported.
- This is the seventh cycle with only hit-rate visible. v003's refactor (early-exit scan, minimal-range rotate, reused miss-scan) is unjudged on its goal. The cost of wider scans in v001 is also unpriced.
- Both v002 and v003 authors could not inspect the code. v003's exact tie confirms policy preservation only if the edit actually took effect.
- The hit-rate levers on this split are essentially exhausted: size, promotion, demotion, ghost state and now associativity all give flat or marginal results.

## Open questions

- Does v003 lower per-op-time, and does v001's wider bucket raise it enough to cancel its small hit-rate gain?
- Is the hit-rate ceiling of this design on the trace reached, so that further gains need an admission or eviction policy of a different kind?
- Do the results hold on traces other than the LIRS-style one?

## Continuation

Stop probing hit-rate knobs in the segmented split (associativity, probation size or sizing rule, promotion, demotion). Treat v001 (wide buckets, slots/16 probation) as the hit-rate base only if per-op-time shows its cost is acceptable. Put per-op-time and metadata-size into the fitness vector first, then judge the v003 rotate refactor against it. After that, try either a different policy family or robustness checks on other workloads.
