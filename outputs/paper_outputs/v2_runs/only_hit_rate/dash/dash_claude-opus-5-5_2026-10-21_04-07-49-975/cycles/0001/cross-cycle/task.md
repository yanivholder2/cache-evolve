# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants changed only the promotion path out of the stash. One changed where an item lands inside its bucket (v001, MRU landing), one changed which bucket it goes to (v002, spill to a free probing slot), and one added an admission filter (v003, two stash hits before promotion). The winner was v003, the only one that made it harder to get into the normal tier. v002 gained very little, and v001 lowered hit-rate.

## Patterns

- What decides the hit-rate here is which items are allowed into the scarce normal tier, not where they are placed. The variant that made admission stricter won. The variant that made it looser (v001, where promoting to MRU lowers the bar for moving to probing) lost.
- Every eviction in the normal tier starts a cross-bucket demotion cascade into a neighbour's target. Changes that cause more normal-tier moves hurt neighbouring buckets as well as their own.
- In steady state the normal buckets are almost always full together. Changes that only matter when a free slot exists (v002) rarely take effect.
- The stash is larger than the normal tier, so it can hold unproven keys while they wait. That is why a filter at the stash→target edge costs little.
- Item position and promotion thresholds are coupled. The "is hottest" check reads bucket position, so changing where an item is placed also changes how many hits it needs for the next promotion.

## Open questions

- Every variant was scored on hit-rate only. Nothing is known yet about metadata-size or per-op-time, including v003's claimed get-latency improvement.
- Is a threshold of two stash hits the best value? Would a frequency or recency filter on the target→probing edge help in the same way?
- Could an SLRU-style MRU landing work if it were combined with a separate hit count, so the bar for entering probing stays where it is?

## Continuation

Build on v003. Try stricter or adaptive admission on the other edges: a different stash-hit threshold, and a hit-count requirement for target→probing. Get v003 evaluated on per-op-time and metadata-size too. Drop "free slot" placement ideas like v002. Only revisit MRU landing if it keeps the current bar for entering probing.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
