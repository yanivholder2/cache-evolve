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
