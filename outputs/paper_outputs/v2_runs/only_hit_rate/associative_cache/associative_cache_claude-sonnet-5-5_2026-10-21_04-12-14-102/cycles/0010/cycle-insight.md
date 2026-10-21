# Cycle 0010 Insight

## Cohort outcome

Three probes were tried on the wide skip-move split: a larger slots/8 probation fraction (v001), a further doubling of associativity to 8x baseline with proportional probation (v002), and eviction of the first never-reused probation entry using a one-bit reuse flag (v003). Only v002 was a Winner, with a very small hit-rate gain. v001 and v003 lost slightly against their parent.

## Patterns

- Every lever is now flat or negative. The third associativity doubling gave only a marginal gain, and the increments keep shrinking. Remaining misses look capacity- or recency-driven, not conflict-driven.
- slots/8 probation is slightly worse than slots/16 against the incumbent. This closes the open probation-fraction question from cycle 0009: extra probation room takes protected capacity and gains nothing.
- A binary reuse bit is a worse eviction signal than plain probation recency. Demoted entries keep the flag, so fresh insertions are evicted before they can prove themselves. This matches the earlier negative results on insertion position.
- Per-op-time and metadata-size are still unmeasured, for the tenth cycle running. The cost of 8x-wide scans against a marginal hit-rate gain is unpriced.
- Authors again could not read the overlay code, so mechanisms are inferred from the hypothesis text and fitness deltas.

## Open questions

- Does the cost of 8x-wide bucket scans cancel the marginal hit-rate gain once time is measured?
- Would a multi-bit or recency-aware signal succeed where the one-bit flag failed, or is the ceiling for this design on this trace already reached?
- Do any of these results hold on traces other than the LIRS-style one?

## Continuation

Stop probing hit-rate knobs in the segmented split: associativity, probation fraction, insertion point, promotion, demotion, and binary reuse flags. Put per-op-time and metadata-size into the fitness vector before running another cycle, then judge v002's width against them. After that, switch to a different policy family, such as frequency-aware admission, or run robustness checks on other traces.
