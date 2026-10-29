# Cycle 0003 Insight

## Cohort outcome

All three variants branched from cycle 0002's adaptive-window winner, and each followed one part of that cycle's continuation: an SLRU main region (v001), cap-free halving counted per `get` (v002), and a climber sample period that does not depend on capacity (v003). Only v001 won. It finally moved web_0 at the large size, where the climber had been stuck. Aging and the faster climber both regressed overall.

## Patterns

- A structural change worked and two timing changes failed. Changing which resident gets evicted (SLRU) added to the adaptive window. Changing *when* things happen (how often the sketch is aged, how often the climber samples) either interfered with the window or did nothing.
- The prior cycle predicted that aging would add to the window. That was wrong. Once the climber gives newcomers room, halving mostly erodes the main region's frequency lead. It hurt at the small size, where it fires often, and had no effect at the large size, where it fires rarely. Fixing the twice-per-miss period removed the suspected cause but not the regression, so the regression comes from the interaction itself.
- The climber's feedback period has to match how fast the cache responds. Because rebalancing is lazy, moving one entry per insert, a window retarget takes about a full turnover to take effect. A fixed short sample scores window sizes that were never actually reached, and the lagging EWMA comparison plus the step floor let the window oscillate. A sample period tied to capacity was too slow at the large size, and a fixed one is too fast at the small size.
- hm_0 stays the hard trace. All three variants dipped it somewhere. Even the winner lost a little at both sizes, because the small probation segment cuts short the stay of recency-driven entries, and demoted stale entries keep inflated counts in a sketch that is never aged.
- Metadata-size and per-op-time are still unmeasured, for the third cycle running. v001 adds a third list and demotion traffic.

## Open questions

- Should the protected fraction adapt, for example climbed like the window or set from probation hit feedback, so that hm_0 recovers without giving back web_0's gain?
- Could aging that targets only demoted protected entries, or aging gated on the climber's state, fix the stale-count problem without the broad erosion seen in v002?
- Would a climber sample period tied to observed rebalance progress (window length reaching its target) or to cache turnover fix both ends of the size range? Should the window also rebalance eagerly instead of lazily?
- Would a different tie-break in the admission contest matter more now that SLRU decides the victims?

## Continuation

Build on v001, the adaptive window plus 80/20 SLRU. The next axis is the protected/probation split, either made adaptive or swept, judged mainly by recovering hm_0. Drop global periodic sketch halving on this lineage, and drop fixed constant climber sample periods. If the climber is revisited, tie its sample period to rebalance completion or turnover, not to a constant. Measure metadata-size and per-op-time before declaring the next winner.
