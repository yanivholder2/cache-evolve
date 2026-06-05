# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants probed independent axes of the Dash promotion ladder — re-entry position, eviction policy, and bucket topology — and all three improved hit-rate with no regressions. The sole Winner, v003, won by rebalancing the normal/stash split to give the hot tier equal standing with cold storage.

## Patterns

- **Topology was the binding constraint.** v003 (50/50 normal/stash rebalance) won without touching any policy logic; simply allocating more hot-tier slots was the highest-leverage move, indicating the baseline over-provisions stash for this workload.
- **Two variants diagnosed the same underlying shortage from different angles.** v001 (MRU re-entry after stash hit) shortens the time an item spends below promotion eligibility; v003 directly expands the space items compete for once eligible. These axes are orthogonal and likely additive.
- **v002's LFU gain is workload-contingent.** The benefit was explicitly tied to workload-a's frequency-skewed access pattern; LRU's recency bias was harming a stable hot set. This is a narrower win than the structural fixes and may not generalise.
- **No variant hurt anything.** The baseline has multiple independent improvement opportunities that do not trade off against each other at this capacity.

## Open questions

- Do v001 (MRU re-entry) and v003 (topology) compound when applied together, or does one subsume the other?
- Is 50/50 the optimal normal/stash ratio for workload-a, or is there a higher-gain split beyond 16/16?
- Does LFU (v002) remain advantageous on less frequency-skewed traces, or does it regress on uniform/recency-heavy workloads?

## Continuation

The hot-tier capacity bottleneck is confirmed as the dominant lever for this cycle; future hypotheses should push the normal/stash ratio further (e.g., 20/12 or 24/8) to find the gain curve's peak or breakpoint. Combining MRU re-entry (v001) with the winning topology is a natural low-risk compound to test. LFU should be held in reserve for workload-specific probing rather than treated as a general improvement.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle probed three directions branching from Cycle 0001's winning topology: compounding MRU re-entry with the 50/50 split, pushing the split further to 75/25, and introducing aging-LFU from a fresh baseline. The sole Winner, v002, confirmed that the normal/stash gain curve had not yet peaked at 16/16 — expanding to 24 normal buckets produced a further lift with no regression, and stash can tolerate reduction to at least 8 slots without penalty.

## Patterns

- **Hot-tier expansion is still the dominant lever.** v002's 75/25 split won outright, reinforcing Cycle 0001's finding that topology is more impactful than policy changes. The working set fits comfortably within 24 normal buckets, meaning capacity was still the binding constraint after the 50/50 rebalance.
- **MRU re-entry and topology are additive but unequal.** v001's compound of MRU re-entry atop the 50/50 geometry improved on v003, confirming the two axes do not interfere, but the marginal gain was modest — once the hot tier is enlarged, collapsing the re-entry penalty matters less because items already have more dwell time.
- **Aggressive per-hit aging is harmful for this workload.** v003's symmetric decay (−1 to all co-resident items on each hit) drove mildly-warm items to eviction before they could contribute hits that recency-ordered LRU would have preserved. Workload-a's access distribution punishes coarse frequency discrimination more than it rewards it.

## Open questions

- Does the normal/stash gain curve continue past 24/8, or does a further push (e.g., 28/4) hit a breakpoint where stash under-capacity starts hurting cold-item re-entry?
- Would stacking MRU re-entry (v001's insertion-position change) onto v002's 75/25 topology produce further additive gains, as it did in Cycle 0001's compound test?
- Is there a gentler aging-LFU formulation — smaller decay increment or access-count-gated decay — that avoids draining mildly-warm items on workload-a?

## Continuation

The topology gain curve is still ascending; the next hypotheses-writer should probe 28/4 and possibly the theoretical limit of maximising normal buckets to locate the peak or breakpoint. Combining MRU re-entry with v002's 75/25 split is the natural low-risk compound to run in parallel. Aging-LFU in its current aggressive form should be dropped; any return to frequency-based eviction should use a finer-grained decay to avoid the mildly-warm eviction problem observed here.


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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle tested three extensions of Cycle 0006's frequency-guard winner: stacking it with second-chance stash recovery, grafting timestamp-based promotion onto it, and raising its eviction threshold. Only v001 — the direct stacking of both prior-cycle gains — won clearly; the other two variants regressed.

## Patterns

- **The two Cycle 0006 gains are additive.** Second-chance stash overflow recovery stacks cleanly with the zero-access eviction guard because the two mechanisms target genuinely distinct pipeline stages: one governs which resident survives eviction within a normal/probing bucket; the other governs what happens to an item expelled from the stash. Combining them closed both loss sites simultaneously without interference.
- **Timestamp promotion is not compatible with frequency-guard eviction.** Selecting the promotion candidate by maximum `last_accessed` rather than by LRU position allows cold items whose creation timestamp is newer than a hot item's last-access timestamp to win promotion. This cold-item promotion leak more than cancels the eviction-guard improvement — the two axes interact destructively, not independently.
- **The zero-access threshold is exactly right for workload-a.** Raising the guard to also evict single-touch items treats them as cold noise, but on this Zipfian workload a single in-tier access is a reliable reuse predictor. Items with one hit belong to the warm working set and should be protected; tightening the threshold shrinks the protected set too aggressively.

## Open questions

- With probing-eviction selectivity and stash-overflow recovery both active, what loss site remains to be closed — geometry (stash capacity), counter staleness, or something else?
- Could timestamp-based promotion be rehabilitated if creation-time contamination were eliminated — e.g., by only recording a timestamp on the first *hit* rather than on insertion?
- Does the `lfu_counter` reset on stash demotion interact beneficially or harmfully with second-chance recovery, given that re-promoted items re-enter the normal tier with a zeroed counter?

## Continuation

v001 — frequency-guard eviction plus unconditional second-chance stash recovery — is the new champion and confirms the stacking hypothesis explicitly recommended by Cycle 0006. The next hypotheses-writer should treat both mechanisms as a fixed baseline and probe remaining loss sites: geometry adjustment (varying stash-to-normal capacity ratio), or a conservative investigation of whether any counter-management policy (e.g., a soft cap on `lfu_counter` to prevent staleness) yields further improvement. The timestamp promotion axis should be deprioritised unless a variant that avoids creation-time contamination can be precisely specified.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle probed three extensions of the Cycle 0007 champion (frequency-guard eviction + second-chance stash recovery): a geometry shift toward a larger normal tier, a change to second-chance insertion position, and a stricter promotion gate. Only v001 — widening the normal/stash split from 3/4 to 7/8 — won, delivering a modest positive lift; the others were neutral-to-harmful or only marginally useful.

## Patterns

- **Normal-tier geometry is still a live lever, but returns are shrinking.** Expanding the normal tier to 7/8 of total capacity continued the upward trend established when the split moved from equal halves to 3/4, yet the absolute gain is meaningfully smaller. The hot set is being captured more completely with each increment, but the marginal value of each additional normal-bucket slot is declining.
- **Second-chance insertion position is a quality gate, not a placement detail.** Inserting stash-evicted items at the MRU end rather than the LRU end of their target bucket caused a clear regression: it granted unproven items immediate promotion candidacy, displacing residents that had earned their position through repeated access. Front (LRU) insertion correctly treats second-chance items as cold probationers, preserving the pipeline's eviction discipline.
- **A stricter single-to-multi-access promotion gate is real but weak.** Requiring two in-target hits before probing promotion filters out a small class of lucky single-hit items, but the workload's hot set has mostly accumulated multi-access evidence before reaching the promotion boundary anyway — leaving little room for this gate to act.

## Open questions

- Is there a natural inflection point in the normal/stash split beyond which the stash becomes too shallow to buffer burst traffic, and the marginal loss from reduced stash headroom overtakes the gain from wider normal capacity?
- Could v001's geometry shift and v003's stricter promotion gate compound, given that they target distinct sites (capacity allocation vs. promotion selectivity)?
- At what split ratio does the frequency-guard eviction mechanism start to lose effectiveness, since a larger normal tier means fewer eviction decisions per unit time relative to the working set?

## Continuation

Geometry (normal-tier width) is now the clearest remaining lever: the next hypotheses-writer should test whether a further split increase (e.g., 15/16 or near-elimination of the probing tier) continues to help or crosses the point where stash shallowness becomes a visible loss source. A targeted combination of v001's geometry with v003's stricter promotion gate is worth one test, as the mechanisms are independent in principle. The Back-insertion path for second-chance recovery is definitively contraindicated and should not be revisited. If geometry fully diminishes, the remaining open axis from prior cycles — soft-capping `lfu_counter` to prevent staleness — becomes the next candidate.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cycle probed two further extensions of geometry (pushing the normal/stash split past 7/8 toward near-elimination of the stash) and one counter-accounting fix (crediting a stash-hit access on the `lfu_counter` before target insertion). Both geometry extensions regressed or were neutral; only the counter-credit change won, delivering a modest positive lift.

## Patterns

- **Geometry has reached its effective ceiling at 7/8.** Widening the normal tier further to 15/16 reversed the gain trend: with only two stash slots remaining, cold-arrival bursts that the parent's four-slot stash absorbed without harm now compete directly with hot-tier residents, and second-chance recovery cannot compensate for the lost headroom. The inflection point is clearly between 28 and 30 normal buckets for this workload.
- **The stricter promotion gate is ineffective at the 7/8 geometry.** Requiring two in-tier accesses before promotion targets a loss site — spurious single-hit promotions competing for scarce probing slots — that only exists when the normal tier is narrow. At 7/8 width, items reaching the promotion boundary are already strongly hot; the gate only delays correct promotions without reclaiming meaningful capacity.
- **Pre-insertion counter credit at the stash exit closed a real but modest protection gap.** Without this change, stash-demoted items re-entering the target tier carry a zeroed `lfu_counter`, making them indistinguishable from cold newcomers under `ClassicLRUFreqProtect` and immediately eviction-vulnerable despite having just been accessed. Crediting the access before insertion eliminates that one-access window. The win is genuine but smaller than prior geometry gains, indicating the vulnerability window existed but was not the dominant loss site.

## Open questions

- With geometry exhausted and the stash-exit protection gap now closed, what is the next meaningful loss site — `lfu_counter` staleness for long-lived hot items, or some structural asymmetry in how the probing bucket handles demotions?
- Could a soft cap on `lfu_counter` prevent established hot items from accumulating stale high counts that crowd out newly warm items, and would that interact positively with the stash-exit credit now in the baseline?
- Does the stash-exit counter credit compound with any remaining counter-management axes, or does it represent the last per-item accounting gain available at this geometry?

## Continuation

The 7/8 geometry is confirmed as the ceiling; the next hypotheses-writer should treat it as fixed and not revisit geometry expansion. v003's stash-exit counter credit is the new champion and should be the baseline going forward. The most promising remaining axis, flagged by multiple prior cycles, is soft-capping `lfu_counter` to prevent staleness in long-lived hot items — this is the primary thread to pursue next. The stricter promotion gate (two-hit requirement) is effectively exhausted at this tier width and should be deprioritised unless paired with a geometry change that makes probing slots scarce again.


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
