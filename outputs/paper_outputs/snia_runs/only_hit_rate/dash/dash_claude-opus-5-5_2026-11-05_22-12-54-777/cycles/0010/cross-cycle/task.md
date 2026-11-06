# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Each of the three variants made one isolated change to the baseline: v001 changed where promotions and demotions are inserted, v002 changed the normal/stash topology, and v003 added a frequency-aware LRU-tail victim choice. All three found real hit-rate headroom. The topology swap (v002, 24 normal / 8 stash) was the Winner because it improved hit-rate on both traces and both sizes, with no regression and no added mechanism.

## Patterns

- All three variants target the same weakness. The baseline's protected tier (target/probing) gives almost no protection, because promoted items are either evicted quickly (Front landing) or crowded out (small normal tier). Each variant fixed a different side of this, and none of them overlap.
- The variants help at different cache sizes. Fixing promotion placement (v001) helped most at the small size. Enlarging the protected tier (v002) helped most at the large size. The frequency filter (v003) helped at both sizes and helped hm_0 more. This suggests the fixes complement each other and are not redundant.
- hm_0 was consistently the more responsive trace, and the small size stayed hard for changes to topology alone.
- v001 showed that moving one lever shifts the dynamics of the others. MRU landing made the probing trigger fire too readily, and at the large size it penalised demoted items on web_0. Stacking these changes may produce similar interactions.
- No variant re-measured metadata-size or per-op-time. The cohort only shows effects on hit-rate.

## Open questions

- Does v002's topology combine additively with v001's MRU promotion and v003's frequency filter, or do they compete for the same gain?
- Is 3/4 normal the best split, or does a more extreme split (e.g. 7/8 normal) keep improving before the stash stops filtering scans?
- What does v003's eviction-window scan and bucket-wide counter halving cost in per-op-time?
- Would a stricter probing-promotion condition remove v001's large-cache web_0 regression?

## Continuation

Use v002's topology as the parent, and stack the other two levers on top of it one at a time: MRU promotion with a stricter probing trigger or a softer demotion position, and the frequency-aware tail victim with stash hits counted. Also sweep the normal/stash ratio past 3/4. Require per-op-time to be measured for any change on the eviction path. Drop further topology-only attempts at the small cache size, since the cache is too tight there for the split alone to help.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants built on cycle 0001's 24/8 topology. v001 stacked MRU-end promotion with a stricter probing trigger and mid-stash demotion. v002 stacked the frequency-aware LRU-tail victim with stash hits counted. v003 pushed the split to 28/4. v002 was the Winner: it was the only variant that improved all four trace/size cells with no regression, and its largest gain came at the small size, where topology alone had not helped.

## Patterns

- Changes that act inside a bucket (which item gets evicted, where a promoted item lands) are what move the small cache size. Topology changes do not. This confirms cycle 0001's call to stop topology-only work at the small size.
- The frequency filter's gain adds cleanly to the topology gain. MRU promotion adds to it on hm_0 but costs a little on web_0. So the two cycle-0001 levers that looked complementary do not stack equally well.
- With admit-everything-to-stash, the stash is the only way into the protected tier, not just a scan filter. Shrinking it starved promotion on reuse-heavy hm_0 before it hurt scan filtering on web_0. The normal/stash ratio has peaked at about 3/4.
- hm_0 is still the most responsive trace in both directions: it shows the largest gains and the largest regression.
- Cost is still unmeasured. No variant re-measured per-op-time or metadata-size, and the Winner adds an eviction-window scan and bucket-wide halving to the eviction path.
- v001 changed three things at once, so its web_0 loss cannot be pinned to one of them.

## Open questions

- What does v002's 8-slot victim scan and halve-on-cap ageing cost in per-op-time, and would a narrower window or a lower cap keep the hit-rate gain?
- Does MRU promotion still add on top of v002's frequency filter, or does the filter already capture the same small-cache gain on hm_0?
- Which part of v001 causes the web_0 loss: mid-stash demotion or the hit-bit probing guard?
- Would a longer stash wait, or promotion on first re-reference, help hm_0's long-gap reuse without growing the stash?

## Continuation

Use v002 as the parent and make per-op-time measurement mandatory. Tune the scan window and counter cap if the cost is high. Then try adding MRU promotion alone, without v001's demotion and trigger changes, and isolate those two pieces only if web_0 regresses. Drop further normal/stash ratio sweeps. Any later gains should come from admission or stash-promotion changes, not from the split.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants made one isolated change on top of cycle 0002's v002 (frequency-aware 8-slot tail victim). v001 moved stash→target landing to the MRU end. v002 put demoted items at the stash MRU end when their counter was 2 or more. v003 narrowed the scan window to 4 and lowered the counter cap to 7. None of them beat the parent. v002 won only because it was essentially neutral: hm_0 did not change and web_0 dropped by a rounding-level amount. The incumbent's hit-rate effectively stalled this cycle.

## Patterns

- Once eviction reads the frequency counter, insert position stops working as a lever. v002's gated demotion was redundant with the tail scan. v001's MRU landing was harmful, not neutral: it moved new arrivals out of the scan window, so the scan evicted established residents instead, and it gave newly promoted items a shorter path to probing. Front landing is part of how the filter works.
- MRU promotion has now hurt web_0 in two cycles. The first time it was bundled with other changes; this time it was applied alone. So the cause is MRU landing itself, not cycle 0002's trigger or demotion changes.
- The 8-slot window does useful work. Narrowing it cost a little on every cell, and the most on hm_0 at the small size, which is where the filter had gained the most. Faster ageing gave no visible benefit in return.
- hm_0's long-gap reuse is still unserved. Changes to recency position cannot reach it.
- Per-op-time has now gone unmeasured for three cycles, even though H03 explicitly asked for it. Every cost/benefit argument about the eviction path is still a guess.

## Open questions

- What does the 8-slot scan plus halve-on-cap actually cost in per-op-time, and does v003's cheaper configuration save anything at all?
- Of v003's two changes, which one caused the hm_0 small-size loss: the narrower window or the lower cap? Testing window 4 with cap 15, and window 8 with cap 7, would separate them.
- Would a wider window (more than 8), or a different ageing rule, help hm_0 more than any change to insertion?
- Would a counter-gated stash→target landing (Back only when the counter is above 1) keep v001's small hm_0 gain without hurting web_0?

## Continuation

Keep cycle 0002's v002 as the effective parent. Drop insert-position changes: no more blanket MRU promotion and no more gated demotion. The next hypotheses should target what the tail scan considers (window width above 8, how counters age) or how much stash capacity there is for hm_0's long-gap reuse. Before trading hit-rate for cost again, the evaluation must actually report per-op-time. Without that, cost-reduction variants like v003 cannot be judged.


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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants built on cycle 0007's v002 (ghost hits go straight to target Front with counter 1). Each one targeted the small-size capacity limit from a different side. v001 gave ghost-admitted keys a one-time exemption from the tail-scan victim choice (protecting credited keys). v002 allowed the bypass only when the target tail window had a free or counter-0 slot (gating on contention). v003 sent counter-0 cascade victims to the ghost instead of the stash (where displaced residents go). v002 was the Winner. It gave the largest small-size hm_0 gain, which nobody predicted, and a small web_0 gain at 16384. It gave back part of the parent's large-size web_0 gain.

## Patterns

- The small-size limit comes from who gets displaced, not from how long credited keys survive. Protecting the newcomer (v001) only moved eviction onto proven residents and hurt every cell, the large size most. Refusing the bypass when every candidate is warm (v002) protected those residents and helped. Fast eviction of un-re-hit ghost arrivals is a useful filter, not a bug.
- Where the cascade victim goes does not matter. Sending it to the stash or to the ghost gave identical hit-rates (v003). Stash churn from cascades is not a lever.
- "No cold slot in the window" is a rough admission signal. It separates warm residents from one-time returners well in a contended small target. In a large web_0 target, tail windows can be all warm, so the gate wrongly blocks returners and sends them back to the stash, where they are lost again, as in c0006. The bypass/stash trade-off now depends on cache size and is not settled by any one rule.
- Two variants' insights could not check their overlay diffs, so their mechanisms are inferred. In particular, v001's broad regression might partly come from a reduced counter cap rather than from the exemption itself.
- Per-op-time and metadata-size are still unmeasured, now for eight cycles. v003's only remaining argument was a cost saving, and it cannot be judged.

## Open questions

- Would a relative gate keep v002's small-size protection without blocking valuable large-size returners? One option: bypass when the returner's credit is at least the window's minimum counter, not only when a counter-0 slot exists.
- On a refused bypass, does keeping the ghost entry (letting a second return retry) actually work, or does it just delay the loss?
- Is large-size hm_0 now flat for structural reasons, with no remaining lever in the ghost/bypass family?

## Continuation

Make cycle 0008 v002 (contention-gated ghost bypass) the new parent. Next, refine the gate condition, for example by comparing the returner's credit with the coldest window counter, to win back large-size web_0 while keeping the small-size gains. Drop protecting credited keys inside the victim scan, and drop changes to where cascade victims are sent. Variant insights should confirm the overlay landed as specified, and per-op-time and metadata-size still need to be measured.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants built on cycle 0008's v002, the contention-gated ghost bypass. Each tried a different way to win back the large-size web_0 loss that v002 caused, without giving up its small-size gains:

- v001 relaxed the gate threshold to admit over counter-1 window minima.
- v002 added the probing neighbour bucket as a second cold-slot placement.
- v003 added a learned per-segment re-hit score as an extra gate branch.

v002 was the Winner, but only because it was the one variant that did no harm. Its gains were noise-level in every cell. In practice the cycle kept the parent and recovered nothing for large-size web_0.

## Patterns

- The single global gate threshold is now used up from both directions. Counter-0-only (c0008) blocks large-size web_0 returners. Counter ≤ 1 (v001) frees them, but it costs more on small-size hm_0, where counter-1 residents are worth protecting. The threshold moves the trade-off between cache sizes. It does not improve both sizes at once.
- More placement options do not help under the same cold-only rule. The neighbour bucket shares the segment and its load, so when the target window is all warm, the neighbour's window is too (v002). The limit is the admission rule, not a lack of nearby slots.
- The learned re-hit signal (v003) did nothing that could be measured. Either the all-warm case rarely triggers it, or "re-hit in target before eviction" is too pessimistic a measure of a returner's value under the demotion cascade. Without counts of how often each gate branch fires, the two cannot be told apart.
- The large-size web_0 gap is real and sits exactly at counter-1 windows. v001 confirmed that all-warm windows at 131072 bottom out mostly at counter 1. The returners worth admitting there compete with counter-1 residents, and on small hm_0 those same counter-1 residents are the ones worth keeping.
- Overlay checks again failed for two of the three variants, so v002's and v003's mechanisms are inferred, not confirmed. Per-op-time and metadata-size are still unmeasured, now for nine cycles. v002's second window read and v003's bitmap both have unknown cost.

## Open questions

- Would a threshold that depends on size or contention keep both sides? For example, ≤ 1 when the target is large or uncontended, and == 0 when it is small or contended, possibly decided from segment occupancy instead of a hard-coded size.
- Does a tie-break that prefers the resident at equal counter keep v001's large-size web_0 gain while removing its small-size hm_0 loss?
- How often does each gate branch actually fire, and how is the v003 score distributed? Instrumentation is needed before any adaptive gate is built on a signal.

## Continuation

Keep cycle 0009 v002 as the parent, or treat 0008 v002 as equivalent, because the gain is noise. Next, try a gate threshold that depends on contention or occupancy: relaxed (≤ 1) only where targets are uncontended, strict (== 0) where they are contended. Separately, try an equal-counter tie-break that favours the resident. Drop placement-spreading (neighbour buckets) and the re-hit-before-eviction feedback score unless they first come with branch-firing instrumentation. Variant insights must confirm the overlay landed, and per-op-time and metadata-size still need to be measured.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

All three variants built on cycle 0009's v002 (the contention-gated ghost bypass). Two took up c0009's open questions on separating the sizes. v001 relaxed the gate only when few tail-window slots were warm, judged by the window's counter histogram. v002 gave a per-key "refused once" strike, so a second return could displace a counter-1 minimum. v003 tried a web_0 lever outside the ghost path: it put sequential-run inserts at the stash's LRU end and suppressed their ghost entries. v002 was the Winner for the second cycle running only because it did no harm. Every cell moved at noise level, and large-size web_0 is still mostly unrecovered.

## Patterns

- The window's counter histogram cannot tell large-size web_0 from small-size hm_0. In both, the tail windows are mostly counter-1. So v001's warm-count guard fired in both places and recreated the c0009 v001 trade-off exactly: large web_0 up, small hm_0 down, mean flat. Any gate that reads only the target window will just move along this trade-off.
- Per-key repeat-refusal evidence is real but too rare to count (v002). The valuable large-size web_0 returners do not mostly go through refuse → stash → drop → ghost → return. Either they return only once within ghost reach, or they get lost in the stash before they fall back to the ghost.
- Sequential web_0 blocks are not one-shot pollution. They are among the ghost's beneficiaries. v003 removed both their stash residency and their ghost entry, and lost hits on web_0 at both sizes and on hm_0 too. This confirms that every web_0 gain in the run has come through ghost readmission, and that removing a key's ghost entry costs hits.
- The ghost/bypass gate family has now stalled for two cycles. Three different gate signals have been tried: a global threshold, window occupancy, and per-key strikes. None improved both sizes.
- For the third straight cycle, overlay checks failed for two of the three variants, so their mechanisms are inferred, not confirmed. No variant has branch-firing counts. Per-op-time and metadata-size are still unmeasured, now for ten cycles.

## Open questions

- What signal about the returner itself separates the two cases? Candidates are how recently its ghost entry was written (its ghost age) or whether it was ever hit in target. A window-only or repeat-only signal does not.
- Do large-size web_0 returners mostly die in the stash before reaching the ghost? If so, the lever may be stash retention for ghost-admitted keys, not the target gate.
- Does v003's scan detector help if only one lever is used: either LRU-end insertion with ghosting kept, or normal insertion with ghost suppression?

## Continuation

Keep cycle 0010 v002, or the equivalent 0009 v002, as the parent. Drop further window-histogram gate variants and the refused-strike credit. Drop sequential-run filters that remove ghost entries. If the gate is revisited, use a returner-side signal such as ghost-entry age. Before building on that signal, first add instrumentation: gate-branch firing counts, and how stash-refused returners end up. Another option is to treat the ghost/bypass line as saturated and try a different axis. Variant insights must confirm the overlay landed, and they must report per-op-time and metadata-size.


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
