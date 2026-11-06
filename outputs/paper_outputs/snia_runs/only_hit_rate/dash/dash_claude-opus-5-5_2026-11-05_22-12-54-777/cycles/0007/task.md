# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0007

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants built on cycle 0002's v002 (frequency-aware 8-slot tail victim, halve-on-cap ageing) and made one isolated change each, targeting what cycle 0003 left open. v001 widened the tail window to 16. v002 added a 64-entry ghost ring per segment, so a key returning soon after being dropped from the stash is re-admitted with counter 1. v003 replaced halve-on-cap with a decrement on every eviction. v002 was the Winner: it was the only variant with a real gain, a small one on hm_0 at both sizes, and web_0 held flat. This is the first change to reach hm_0's long-gap reuse.

## Patterns

- The tail-scan filter's tuning is now mapped from both sides. Window 8 sits at the peak: narrower (cycle 0003) loses the frequency filter, and wider (v001) turns the policy into coarse LFU and loses recency protection. Slow, rare ageing also matters: faster ageing (v003) drains counts before they can protect anything. The filter's parameters are spent as levers.
- The small-cache hm_0 cell keeps showing how the filter works. It is where the filter gained the most, where v001's wider window still helped slightly, and where v003's fast ageing hurt the most. When capacity is tight, frequency matters more than recency there, which suggests the best settings depend on cache size or bucket type.
- Admission-side memory moved hm_0 when recency-position changes never did. It still only reaches reuse gaps just past stash residency, and a one-step bonus is fragile under ageing.
- web_0 again barely responded to anything, either up or down. No lever tried so far has a scan-heavy win.
- Fast ageing failed partly because promoted items land at Front, inside the window, and get aged back to 0. Front landing and counter protection are coupled. Any ageing or bonus change has to account for this.
- Per-op-time and metadata-size have now gone unmeasured for four cycles, even though every hypothesis asked for them. The Winner adds a scan on every `put` and per-segment state, and both costs are unknown.

## Open questions

- Does a larger ghost ring, or a bigger counter bonus on a ghost hit, extend the hm_0 gain, and at what per-op-time cost?
- Would a window size or ageing rate scaled per bucket type (stash vs normal) or by cache size capture v001's small-hm_0 gain without its losses elsewhere?
- Would slower-than-halving ageing (every Nth eviction, or exempting newly promoted items) help, or is the current ageing already near optimal?
- Is there any lever that helps web_0?

## Continuation

Make cycle 0004 v002 (ghost-credited admission) the new parent. Scale its reach: try a larger `GHOST_SLOTS` and a stronger ghost-hit bonus as separate variants, and require per-op-time and metadata-size to be reported, since the ring grows the admission path. Drop fixed window-width sweeps and faster ageing. If ageing or window are revisited, only try per-bucket-type or slower-decay versions.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
