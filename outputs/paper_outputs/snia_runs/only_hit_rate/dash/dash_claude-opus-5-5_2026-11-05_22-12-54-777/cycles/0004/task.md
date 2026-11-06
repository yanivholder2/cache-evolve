# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0004

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0004/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0004/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0004/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0004/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-11-05_22-12-54-777/cycles/0004/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
