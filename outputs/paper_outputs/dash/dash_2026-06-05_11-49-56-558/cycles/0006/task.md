# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle pushed three threads simultaneously: locating the ceiling of hot-tier expansion, compounding MRU re-entry onto the winning 75/25 topology, and retrying frequency-based eviction with a softer threshold gate. Only the compound (v002) won — with a small but clean gain — while the topology push exposed the breakpoint and the gentler aging-LFU still could not match position-based recency.

## Patterns

- **The topology gain curve has peaked.** Extending to 28/4 caused a regression, confirming that 24/8 is the optimal normal/stash split for workload-a. With only four stash slots, cold items cannot be held long enough to re-enter on their next access, and the added hot-tier pressure costs more than it returns. The geometry-only lever is now exhausted.
- **MRU re-entry is additive but nearly subsumed by topology.** Stacking Back-insert onto the 75/25 geometry produces a genuine, positive increment, but it is modest — because the enlarged normal-bucket pool already provides sufficient dwell time for natural promotion. The two axes do not interfere but they are not strongly synergistic either.
- **Position-based recency consistently beats frequency discrimination on this workload.** Threshold-gating the co-resident decay rescued co-residents from aggressive drain yet still underperformed ClassicLRU. The slow counter ramp-up creates its own eviction blind spot in early item life, replacing one failure mode with another. Item position encodes a stronger future-access signal than any frequency counter variant tested so far.

## Open questions

- With hot-tier topology exhausted and re-entry gains marginal, what genuinely new axes remain? Ghost/victim caches, admission control, or scan resistance have not been probed.
- Is there a re-entry formulation more nuanced than full Back-insert — e.g., inserting proportionally based on access count — that could extract more from the position lever?
- Would frequency-based eviction ever win on workload-a, or is the access pattern so recency-dominant that it is structurally incompatible?

## Continuation

The two previously dominant levers — hot-tier topology and MRU re-entry — are now close to fully exploited on workload-a; future hypotheses should pivot to unexplored structural axes such as ghost/victim-cache admission control or scan-resistant insertion policies. Frequency-based eviction in any form should be retired for this workload until evidence from a different workload trace motivates revisiting it. The confirmed optimal geometry (24/8) and the MRU re-entry policy together form the new compound baseline from which the next round of probes should launch.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants probed structural axes around stash admission and promotion gating — ghost-based bypass, LFU-gated promotion, and stash-free direct insertion — in line with Cycle 0003's call to explore unexplored terrain. Only v002 (ghost re-admission) won, and only by a margin that is effectively noise; the other two variants regressed.

## Patterns

- **The stash is a load-bearing quality gate, not just a capacity buffer.** Both attempts to weaken or circumvent it — front-door bypass on cold insertion (v003) and threshold-gating the hot-item promotion pipeline (v001) — degraded hit-rate. Removing or slowing stash traversal allows cold or slow-to-qualify items to displace established warm residents, which costs more than the saved hop recovers. This role as admission filter appears fundamental for workload-a's stable hot set.
- **Ghost re-admission is safe but near-neutral.** The ghost set (v002) produced only a marginal, noise-level improvement: the workload's reuse pattern does not cycle through the stash boundary frequently enough for bypass tracking to yield a real benefit. The mechanism is correct in principle but mismatched to this workload's access characteristics.
- **Promotion gating introduces friction with no real payoff.** The LFU threshold in v001 was designed to filter one-time scans from the probing tier, but scan pollution was not a meaningful loss source here — probing slots were already being used productively. Adding a counter-gated delay simply slows the hot-item pipeline.

## Open questions

- With topology, re-entry, admission control, and promotion gating all explored or exhausted, what genuinely novel levers remain for workload-a? Scan-resistant insertion *within* the normal tier (rather than at the stash boundary) has not been probed.
- Would any of these admission-control ideas produce real gains on a workload with heavier scan or bursty cold-item arrival patterns, where the quality-gate role of the stash is under more stress?
- Is the ghost set's near-zero gain truly a ceiling, or does it depend on the ghost set size and Front-insertion position chosen for re-admitted items?

## Continuation

The stash-as-admission-filter finding should be treated as a constraint: next-cycle hypotheses must not weaken or bypass stash traversal on new insertions. The ghost set (v002) can be set aside — it is safe but contributes nothing to workload-a. The improvement space for this workload is now quite narrow under current structural axes; the next hypotheses-writer should consider either intra-normal-tier eviction ordering (e.g., scan resistance or segmented LRU within the hot tier), or shifting to a second workload trace to reveal whether a different access pattern unlocks frequency-based or admission-control gains that are structurally blocked on workload-a.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants probed insertion-position ordering — ghost re-admission positioning, a canonical promotion/demotion asymmetry in the normal tier, and a probationary zone in probing — and all three regressed relative to the parent. v001 won by being the least harmful: ghost re-admission at MRU produced only marginal noise, while the other two incurred clear regressions.

## Patterns

- **MRU positioning dominates universally on workload-a.** Every attempt to withhold MRU status — whether at the ghost re-admission boundary, the demotion cascade, or the probing entry point — hurt hit-rate. The workload's reuse gaps are short enough that displaced or newly-promoted items reliably get re-accessed before eviction, making probationary or LRU-entry positioning consistently punishing rather than protective.
- **The ghost set is now exhausted as a lever in all configurations.** Back-insert on re-admission moved the needle marginally in the wrong direction, confirming the earlier finding that ghost re-admission delivers noise-level movement regardless of position. The stash eviction boundary simply is not where meaningful hit-rate is lost on this workload.
- **Textbook segmented-LRU asymmetry (promotions at MRU, demotions at LRU) is contraindicated here.** The standard design assumes a clean hot/cold divide, but workload-a's access distribution is warm throughout — items displaced by a hot promotion still have residual heat, and inheriting MRU position in the destination tier is what preserves their next hit. Removing that buffer accelerates churn rather than improving selectivity.

## Open questions

- With MRU universally dominant, is there any meaningful insertion-ordering axis left to probe, or has this design axis been fully exhausted?
- Would workload-a behave differently under a capacity stress test (smaller total cache size), where probationary filtering might finally pay off because good eviction decisions matter more at the margin?
- Is there a genuinely new structural mechanism — adaptive segment sizing, size-aware eviction, or TTL-based decay — that workload-a's stable hot set would respond to?

## Continuation

The ghost set should be permanently retired; it has been shown noise-level across multiple position and size configurations. The universal MRU finding should be encoded as a constraint: any future variant that inserts at LRU/Front for newly-arriving or re-admitted items is likely to regress on workload-a. The search space under the current design appears close to exhausted; the next hypotheses-writer should either introduce a structurally new mechanism (adaptive sizing, eviction cost weighting) or switch to a second workload trace where cold-item churn or scan traffic would make probationary and frequency-aware schemes genuinely competitive.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
