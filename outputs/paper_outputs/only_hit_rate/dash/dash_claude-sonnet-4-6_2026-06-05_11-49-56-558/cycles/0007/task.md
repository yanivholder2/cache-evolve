# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0007/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants probed distinct new axes — eviction selectivity within a tier, unconditional stash-eviction recovery, and timestamp-driven promotion — and all produced some improvement over their respective parents. v001, which guards confirmed-reuse residents from eviction by zero-access newcomers, won clearly; the second-chance and timestamp variants improved but did not match it.

## Patterns

- **Eviction selectivity is a live lever even after topology and insertion-position are exhausted.** Preferring to evict items that have never been accessed within their current tier over items with at least one confirmed hit produces a clear gain. The probing boundary turns out to be a meaningful loss source: newly promoted items with no in-tier reuse had been displacing established residents on workload-a's working set.
- **Second-chance promotion definitively outperforms the ghost set.** Unconditionally moving every stash-evicted item to the LRU end of its target bucket works better than the ghost set's conditional, event-driven re-admission — confirming that the ghost set's weakness was its dependency on a future external re-insertion event, not the insertion position chosen. The second-chance mechanism's gain is real but smaller than the frequency-guard gain, suggesting eviction selectivity was the bigger bottleneck.
- **TimestampLRU replicates established gains without surpassing them.** Decoupling promotion candidacy from insertion order (so a freshly stash-promoted item does not automatically become the next promotee) provides benefit under the Zipfian distribution, but this effect is already partially captured by the confirmed-topology and MRU-insertion combination from earlier cycles.

## Open questions

- Can v001's frequency guard be stacked with v002's second-chance recovery for a compounded gain, since they address different loss sites (probing eviction selectivity vs. stash overflow recovery)?
- Is the zero-access threshold in the frequency guard the right cutoff, or would a slightly higher threshold (e.g., evict items with fewer than two in-tier hits) yield further improvement?
- Does the frequency counter reset on stash demotion interact with the second-chance mechanism in a useful or harmful way if both are active simultaneously?

## Continuation

v001's frequency guard is the new champion and the most significant improvement in several cycles. The next hypotheses-writer should prioritise stacking v001 with v002's second-chance mechanism to test whether the two gains are additive — they target different parts of the pipeline (probing eviction vs. stash overflow). If that compound wins, a further probe of the frequency threshold (counts > 0 vs. counts > 1) is a natural follow-on. TimestampLRU (v003) need not be abandoned but should be tested as an addition to the winning frequency-guard baseline rather than as a standalone replacement.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
