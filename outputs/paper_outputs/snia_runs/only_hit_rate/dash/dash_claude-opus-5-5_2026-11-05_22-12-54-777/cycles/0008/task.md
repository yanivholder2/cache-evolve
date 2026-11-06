# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0008

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants built on cycle 0004's v002 (64-slot ghost ring, admission at counter 1). Each one isolated a single factor behind the ghost's limited reach on hm_0. v001 replaced the ring with a direct-mapped ghost table sized to the segment's capacity (reach). v002 stored the dropped counter and re-admitted at remembered+2, using 12-bit fingerprints (bonus strength). v003 removed the bucket-wide halving on stash hits (ageing drain). v001 was the Winner: it gave a clear gain on small-size hm_0, the largest ghost gain so far, with no regression anywhere. It also made ghost lookup O(1) instead of a per-`put` scan.

## Patterns

- Reach is what limits the ghost, not bonus strength. A cache-sized ghost clearly beat the ring. The stronger bonus helped only the same small-hm_0 cell, and only a little. Stash victims come from the low-counter tail, so the "remembered" count is usually near zero anyway.
- Ghost gains stay concentrated in small-size hm_0, the same cell where the frequency filter has always done the most. At the large size, more reach and a bigger bonus both did nothing. The reuse still lost there is either beyond one cache's worth of drops or not saved by a single admission step.
- web_0 stays inert to ghost reach. It reacted only to v002's narrower fingerprint, with a small dip from false matches or stickier admissions. Fingerprint width is a real cost, and scan-heavy traces are where it shows.
- Halving on a stash hit is not an incidental drain. It is the stash's only ageing, and it also limits how high a counter a re-promoted item carries back into target. Removing it hurt every cell, the large size most. This is the third cycle confirming that counters must keep ageing, and that Front landing plus high counters makes items stick in target.
- Per-op-time and metadata-size are still missing from the fitness vector, now for five cycles. The Winner's cost advantage, a scan replaced by an O(1) lookup, is argued from the code and has not been measured.

## Open questions

- Does a stronger bonus add on top of v001's cache-sized ghost, given that the ghost now fires more often? If it does, it needs to keep full-width fingerprints.
- What reaches the large-size hm_0 reuse: a ghost larger than the cache, or a different admission rule?
- Would ageing the stash at a different point (on insertion, or resetting/halving a demoted item's counter on entry into the stash) protect ghost credits without v003's stickiness?
- Is there any lever at all for web_0?

## Continuation

Make cycle 0005 v001 (direct-mapped, capacity-sized ghost) the new parent. Next, try a bonus of 2 on it without cutting fingerprint bits, and try a ghost sized above capacity (for example 2×) for the large-size hm_0 cells. Drop removing the stash-hit halving, and drop any change that narrows the fingerprint. If stash ageing is revisited, move it rather than remove it.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants built on cycle 0005's v001 (a direct-mapped ghost sized to the cache, admitting at counter 1). Each one isolated a different explanation for why large-size hm_0 stays stuck. v001 raised the ghost-hit credit to 2 (bonus strength). v002 doubled the ghost table (reach). v003 halved an item's counter when it was demoted into the stash (where ageing happens). v003 was the Winner. It was the only variant with a small, consistent hm_0 gain at both sizes, including the first movement at the large size, and web_0 was flat to noise-level negative.

## Patterns

- Ghost reach and ghost bonus strength are now both used up. A credit of 2 barely moved small-size hm_0, because a credit of 1 already beats the mostly counter-0 stash tail. A 2× table did nothing at the large size and slightly hurt the small size, because older returners diluted the bonus. Neither the bonus nor reach is what limits large-size hm_0.
- What helped was changing who the ghost-credited arrivals compete against, not making the credit itself bigger. Taking away the stale target-tier counts that demoted items carried into the stash helped. This is the fourth cycle in which counter ageing turned out to be the active lever. It is also the first time ageing was added at a point where no hit is penalised, and that is why it was safe, unlike removing the stash-hit halving.
- Gains still depend on the ghost firing. That is why hm_0 responds and web_0 does not. web_0's only reactions this cycle were noise-level dips: demoted items returning to target with less protection (v003), and over-credited scan keys (v001).
- The effects are getting smaller each cycle. Every lever in this family (window, ageing, ghost reach, bonus, ageing point) now yields small gains, which suggests the current structure is close to its limit.
- Per-op-time and metadata-size are still not in the fitness vector, now for six cycles. v002's extra metadata was reported only as an estimate.

## Open questions

- Do other tier boundaries also benefit from moving ageing there? For example: halving on stash→target promotion, or ageing applied inside the normal tier's tail scan.
- Does large-size hm_0 lose reuse through exit paths that never reach the ghost (keys leaving other than by stash drops)?
- Would a stronger demotion decay (a quarter, or a reset) help hm_0 more, or start hurting web_0 at the large size?
- Is there any lever at all for web_0, perhaps scan detection or admission filtering on the normal tier rather than the stash?

## Continuation

Make cycle 0006 v003 (halve on demotion into the stash) the new parent. Drop further ghost-size and ghost-bonus tuning; both are ruled out. Next, test where ageing is applied at other boundaries (promotion back into target, normal-tier eviction), and check whether hm_0's lost keys leave through paths the ghost never records. Given the shrinking gains, consider one structurally different hypothesis aimed at web_0, and push again for per-op-time and metadata-size to be measured.


### Cycle 0007

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
