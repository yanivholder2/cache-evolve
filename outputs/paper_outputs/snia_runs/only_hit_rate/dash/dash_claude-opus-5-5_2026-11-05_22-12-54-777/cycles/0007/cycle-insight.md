# Cycle 0007 Insight

## Cohort outcome

All three variants built on cycle 0006's v003 (halve on demotion into the stash). Each tried a different lever. v001 added a CLOCK-style decrement to the normal-tier tail survivors on every eviction (ageing at a new boundary). v002 sent ghost hits straight to target Front with counter 1 instead of back into the stash (where credited keys land). v003 added a TinyLFU admission gate on stash→target promotion (aimed at web_0). v002 was the Winner. It gave the largest hm_0 gain the ghost line has produced, plus the first real web_0 gain in the run, both at the large size. The small size was neutral to slightly positive.

## Patterns

- Where a credited key lands mattered more than how much credit it gets or how far the ghost reaches. Ghost reach (c0005) and bonus size (c0006) stalled because credited keys went back into the short-lived stash and were lost again before their next reuse. Skipping the stash turned the credit into target-tier residency. The constraint was structural, and it could not have been found by tuning parameters.
- web_0 finally moved, but through the ghost, not through scan defence. At the large size the ghost catches enough web_0 returners for the bypass to pay off. A gate aimed directly at scans (v003) did nothing: one-hit scan keys never get a stash hit, so they never reach the gate, and target-tail counters sit near 1, so the gate rarely rejects. Whether a cell responds still depends on whether the ghost fires.
- Ageing is safe only when it targets items already judged cold. v003's demotion halving (c0006) did this. v001's decrement on every eviction did not. It hit the Front-landed counter-1 returners inside the tail window, hurt hm_0 most at the small size, and dipped web_0 at both sizes. This is the fifth cycle showing that Front landing and ageing are coupled. Ageing triggered by eviction count also hits small caches hardest.
- Gains from the Winner scale with target-tier capacity. At the small size, counter-1 Front arrivals are evicted before they can be re-hit, and ghost-driven demotion cascades in a contended target can swap out useful residents.
- Per-op-time and metadata-size are still unmeasured, now for seven cycles. The Winner moves ghost-hit puts onto the target-insert and cascade path, and that cost is unknown.

## Open questions

- At the small size, would ghost-admitted keys survive longer in target if they landed slightly deeper than Front, or were briefly exempt from the tail scan?
- Should only some ghost hits bypass the stash, for example only at large sizes or only for keys with a recent ghost entry, so that small-size web_0 avoids harmful cascades?
- Are there other paths where proven-reuse keys are still sent through the stash, such as demoted items with prior target hits, that could also benefit from a direct placement rule?

## Continuation

Make cycle 0007 v002 (ghost hits admitted straight to target Front, counter 1) the new parent. Next, explore how ghost-admitted keys are placed and protected in target, especially to address the small-size capacity limit. Drop eviction-count-driven normal-tier ageing, unless it excludes recently promoted slots. Drop promotion-side admission gates. Insist on per-op-time and metadata-size measurements.
