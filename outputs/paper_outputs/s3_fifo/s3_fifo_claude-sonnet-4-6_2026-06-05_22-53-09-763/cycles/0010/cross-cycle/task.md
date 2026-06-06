# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants targeted the balance between the small, main, and ghost queues; only the pure capacity reallocation (shrinking the small queue from 25% to 10%) won, while both attempts to loosen admission quality — lowering the promotion threshold and widening the ghost window — hurt hit rate.

## Patterns

- **Resident capacity beats admission breadth.** The winning change gave more slots to main without touching any admission logic. Both losing changes expanded what could enter main (more promotions, more ghost hits fast-pathed in) and degraded performance, suggesting main queue quality is more sensitive to entry criteria than to raw size pressure.
- **The main queue's second-chance mechanism is the critical bottleneck.** When lower-quality entries flooded in (v002, v003), second-chance eviction could not shed them fast enough to protect genuinely hot items. The frequency threshold (`freq > 1`) and a right-sized ghost window act as upstream guards that second-chance alone cannot replace.
- **Ghost queue value is precision, not coverage.** Extending the ghost window introduced incidental re-accesses that bypassed the small-queue filter. A longer history does not improve signal quality for this trace; it introduces noise from distant, non-recurring accesses.

## Open questions

- Is 10% the right floor for small queue size, or does further shrinking (e.g., 5%) keep improving — and does the answer change under scan-heavy or more bursty phases?
- Could a *smaller* ghost queue (below the baseline `main_size` cap) improve precision further, given that a larger one demonstrably hurts?
- Does the winning small=10% configuration shift the optimal promotion threshold, or does `freq > 1` remain correct regardless of small-queue sizing?

## Continuation

Next cycle should build on v001's winning configuration (small=10%) and probe in two directions: first, whether fine-grained small-queue sizing around 10% (e.g., 8%, 12%) yields further gains; second, whether the ghost queue has an optimal size below the current baseline cap — the evidence from v003 suggests shrinking it is worth testing. Attempts to relax the small-to-main promotion threshold should be parked unless paired with a compensating mechanism to protect main-queue quality.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants probed two axes independently and in combination: further small-queue shrinkage and ghost-queue precision via a halved cap. The winner (v003) stacked cycle 0001's enlarged main queue with a tighter ghost window, confirming that both dimensions contribute independently to hit rate.

## Patterns

- **Ghost precision is a first-class lever.** Both variants that halved the ghost cap (v002 on the baseline, v003 on the cycle-0001 configuration) improved hit rate — the ghost truncation worked regardless of which small-queue ratio accompanied it. The benefit is not an artifact of the enlarged main queue; it is a property of the workload's re-use distance fitting inside a smaller window.
- **The two levers are complementary, not redundant.** v003's combined change outperformed v001 (small-only) and, by extension, the cycle-0001 winner. A larger main queue and a sharper ghost signal amplify each other: the enlarged main slows eviction churn, making the freshest evictions colder on average and therefore making ghost precision matter even more.
- **Small-queue shrinkage is hitting diminishing returns.** The 10% → 8.3% step in v001 produced a gain visibly smaller than the large 25% → 10% jump from cycle 0001. The marginal value of each slot transferred from small to main is declining, signalling a plateau in this direction.

## Open questions

- Does the ghost benefit continue below main_size/2 (e.g., main_size/4), or does the window become too narrow and miss genuine re-uses?
- With v003 as the new baseline, does further small-queue reduction (e.g., capacity/12 or capacity/14) still help, or have we exhausted small-queue tuning?
- The promotion threshold (`freq > 1`) has remained constant throughout; with a sharper ghost signal now filtering main-queue entry, is there headroom to explore `freq > 0` without flooding main?

## Continuation

Next cycle should treat v003 as the new baseline and probe ghost sizing more aggressively — specifically a ghost cap of main_size/4 — to find where ghost truncation tips from precision gain to signal loss. A secondary axis worth opening is whether the combination of the sharper ghost and the now-stable small-queue size creates room to revisit the promotion threshold, which has been untouchable until now. Further small-queue-only shrinkage is unlikely to be the highest-yield direction given the diminishing returns seen in v001.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants probed distinct structural axes — ghost compression, promotion threshold relaxation, and second-chance credit depth — and none improved hit rate; v002 (freq > 0 threshold) was the least harmful and is the nominal winner, but the cycle effectively confirms boundaries rather than advances the configuration.

## Patterns

- **All three axes hit limiting cases simultaneously.** Halving the ghost cap again crossed below the workload's re-use horizon (v001); widening the promotion gate was safe but found no eligible traffic to benefit from it (v002); tightening MAX_FREQ cut off third-pass retention that live hot items actually exercised (v003). The cohort collectively mapped the outer edges of three independent knobs in a single cycle.
- **main_size/2 is a hard floor for ghost capacity on this workload.** The re-use distances in the trace extend into the 461–922-eviction range; any ghost window shorter than main_size/2 discards genuine signal, not noise. This boundary is now experimentally confirmed from both sides (expanding hurt in cycle 0001; shrinking hurt here).
- **MAX_FREQ = 3 is the functional lower bound for second-chance depth.** The hot set is temporally sticky enough that some fraction of items legitimately earns and uses a third eviction-pass reprieve. This also establishes that second-chance depth has unexplored upward space (≥ 4).
- **The freq > 0 admission gate is safe but inert alone.** The narrow small queue prevents a quality flood, but the workload simply does not carry enough once-accessed-then-reused traffic to translate wider admission into observable gains. This change is not harmful and could be revisited in combination with other structural changes.

## Open questions

- Does raising MAX_FREQ above 3 (e.g., 4 or 5) retain additional hot-set items long enough to improve hit rate, or does it instead delay eviction of moderately warm items and hurt?
- With the freq > 0 admission confirmed neutral, could it interact positively with a higher MAX_FREQ — admitting more items into main while giving them a longer retention window?
- Are there axes entirely unexplored so far (e.g., eviction ordering within main, hit-count reset on ghost re-admission) that could yield gains now that queue sizing and threshold knobs are largely exhausted?

## Continuation

The cycle 0002 winner remains the best-known configuration; next cycle should treat it as the baseline and stop revisiting ghost sizing or small-queue ratios, which are well-characterised. The most promising unexplored axis is MAX_FREQ upward (≥ 4): the evidence that 3 is a lower bound implies the upper direction is worth probing. A secondary experiment worth attempting is combining the now-safe freq > 0 promotion threshold with a higher MAX_FREQ, since the two changes interact structurally — wider admission into main is more viable when retention depth gives newly admitted items time to prove their worth.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants probed retention depth and admission quality: raising the second-chance frequency ceiling (v001), widening small-queue admission (v002), and granting ghost-re-admitted items a head-start credit (v003). Only the ceiling raise won, and the gain was real but modest.

## Patterns

- **MAX_FREQ = 4 yields a genuine but incremental gain.** A non-trivial minority of items reach the old ceiling and benefit from the extra retention pass, confirming the upward direction is viable — but the small magnitude of the improvement suggests this minority is limited, and proportional returns from further ceiling increases are unlikely.
- **The freq > 0 admission gate is definitively closed.** Tried twice (neutral solo in cycle 0003, harmful here in combination with ghost precision), it never helps. Diluting main with single-access items costs more in churn than the marginal ghost-hit promotions can recover; the two mechanisms are orthogonal rather than synergistic on this workload.
- **Ghost re-admission credits are counterproductive.** Seeding re-admitted items at freq=1 displaces incumbents that earned their slots through genuine accesses. The ghost signal confirms re-use potential, but not enough to justify claiming a residency advantage over established main-queue residents.

## Open questions

- Is there a useful stopping point between MAX_FREQ = 4 and a higher ceiling (e.g., 5 or 6), or does the frequency distribution of hot items make any further step negligible?
- With admission thresholds and ghost sizing now well-characterised, are there structural axes entirely unexplored — such as eviction ordering within main, hit-count handling on ghost re-admission at freq=0, or adaptive queue sizing under phase changes?

## Continuation

Next cycle should adopt MAX_FREQ = 4 as the new baseline and make one cautious test of MAX_FREQ = 5, but with low expectations given the already-small delta from 3 → 4. The admission threshold and ghost credit directions are exhausted and should not be revisited. If the MAX_FREQ = 5 probe is also incremental or flat, the highest-yield path forward is likely a genuinely new structural axis — the queue-sizing and admission-threshold knobs have been mapped to their effective limits.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle probed three independent axes — frequency-ceiling extension, eviction-ordering strategy within main, and promoted-item frequency normalization — yielding one tiny positive (v001), one significant regression (v002), and one neutral result (v003). The winner advances the configuration by the smallest margin seen across all cycles.

## Patterns

- **The frequency-ceiling axis is now exhausted.** The 4→5 increment produced an even smaller gain than 3→4, and the v001 insight characterises the hot-set tail as sparse above the old ceiling. Each step yields a shrinking return; further increments are not a productive direction.
- **Move-to-front reprieve in main eviction is load-bearing recency sorting, not merely a second-chance gate.** The CLOCK-style in-place scan tried in v002 revealed that the baseline's positional churn continuously keeps recently re-accessed items away from the eviction frontier. Removing that implicit recency re-ordering — even while preserving the frequency check — caused a large regression. Eviction ordering is a first-class dimension of this cache's performance.
- **Promoted-item frequency normalization is inert on this workload.** Capping incoming main-queue items at freq=1 had no measurable effect, indicating that over-credited promotions are not displacing established hot residents in practice. This axis can be set aside.

## Open questions

- Since move-to-front is the primary recency-sorting mechanism, are there modifications to that behaviour (e.g., conditional repositioning above a frequency threshold, or partial repositioning) that could improve hit rate further without discarding the recency benefit?
- With queue sizing, admission thresholds, ghost sizing, frequency ceiling, and promotion normalization all characterised, what genuinely unexplored structural axes remain? Candidates include adaptive queue sizing under workload phase shifts, eviction tie-breaking within frequency bands, or behaviour specific to scan-heavy versus hot-set-heavy access phases.

## Continuation

Both the frequency-ceiling and promotion-normalization axes are now closed. The most actionable finding from this cycle is that eviction ordering within main is the most sensitive structural lever yet identified — the implicit recency sort embedded in move-to-front is the real mechanism behind the baseline's performance. The next cycle should treat this as the primary axis: explore whether selective or conditional repositioning during eviction can improve on the baseline's unconditional move-to-front, rather than continuing to chase diminishing returns on the well-mapped parameter knobs.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

All three variants probed fine-grained positional and threshold adjustments within main — tightening the second-chance gate, repositioning saturated items proactively, and varying the insertion point of newly promoted items — and produced the smallest aggregate movement of any cycle to date: two negligible positives and one true zero.

## Patterns

- **Positional manipulation within main is inert on this workload.** Both the saturation-triggered head-repositioning (v002) and the midpoint-insertion variant (v003) found that the hot set almost never satisfies the combination of conditions required to benefit: items that reach the frequency ceiling rarely sit close enough to the eviction tail for a position rescue to matter, and the eviction scan recycles them adequately without physical relocation. Fine-grained placement is not a lever.
- **The freq=1 population is too small and cold to be worth gate-keeping.** Tightening the second-chance threshold moved the metric in the predicted direction, but the magnitude confirms that freq=1 traffic is a negligible fraction of re-access candidates on this workload. The hypothesis was correct in principle but irrelevant in practice.
- **The configuration is approaching a local optimum.** The cohort's combined effect on hit-rate is barely measurable. All axes probed across six cycles — ghost sizing, admission thresholds, frequency ceiling, ghost credits, eviction ordering, and now positional tuning — have been mapped to diminishing returns or outright failure.

## Open questions

- Is the current configuration genuinely near-optimal for workload-a, or is there a structural axis that hasn't been framed yet — such as adaptive queue sizing under phase changes, scan-burst detection, or workload-specific eviction tie-breaking?
- Given that move-to-front is the dominant recency mechanism and positional refinements don't help, could *conditional* move-to-front (e.g., only reposition above a frequency threshold) extract any remaining signal without regressing to the cycle-0005 failure mode?

## Continuation

The static-parameter and positional-tuning search spaces are exhausted; further incremental adjustments to thresholds, insertion points, or ceiling values are unlikely to yield meaningful gains. The next cycle's hypotheses-writer should either commit to a genuinely new structural axis — adaptive or workload-phase-sensitive mechanisms are the most plausible candidates — or explicitly declare the configuration converged and benchmark it against an alternative algorithm rather than continuing to refine knobs within the current design.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle probed three genuinely structural axes — eager small-queue promotion, frequency-tournament eviction selection, and adaptive threshold control — representing the first cohort to move beyond static parameter tuning. None of the variants improved hit rate: the two winners were null results, and the one that tried dynamic adaptation produced a slight regression.

## Patterns

- **Structural hypotheses confirm the baseline is robust, not just locally tuned.** Eager promotion and tournament selection both targeted theoretically real failure modes (small-queue lag, frequency-position inversions) but found those modes are already suppressed by the existing design on a Zipfian workload. The mechanisms are correct in principle but have nothing to act on in practice.
- **Adaptive control requires workload diversity to function.** The phase-sensitive threshold mechanism hurt precisely because workload-a is phase-stable: the signal the controller watches never moves meaningfully, so the rare adaptation event fires on noise rather than a real phase shift. This is a workload-suitability failure, not a design failure.
- **The null results are collectively diagnostic.** Across seven cycles, every axis — ghost sizing, admission thresholds, frequency ceiling, ghost credits, eviction ordering, positional tuning, and now promotion timing and tournament selection — has been either exhausted or confirmed inert on workload-a. The configuration is not drifting toward a local optimum; it appears to have reached it.

## Open questions

- Would the adaptive threshold mechanism (v003) perform as hypothesised on a workload with genuine scan bursts or hot-set phase transitions, or does the control surface need to be wider than three levels?
- Is the remaining performance gap between this configuration and a theoretically optimal cache explained by workload-a's inherent access entropy, or is there a structural property of S3-FIFO that a different algorithm (e.g., TinyLFU, SIEVE) would exploit?

## Continuation

The search space within S3-FIFO on workload-a is exhausted. The next cycle's hypotheses-writer should not introduce further structural refinements to the current algorithm — all plausible levers have been mapped. The productive paths forward are: (1) benchmark the current best configuration against an alternative algorithm on workload-a to characterise the residual gap, or (2) run the most promising adaptive variant (v003) against a phase-shifting or scan-heavy workload to determine whether the mechanism has value in a context where it can actually operate. Continuing to refine S3-FIFO on workload-a will not yield meaningful gains.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle revisited three axes — second-chance threshold tightening, small-queue size reduction, and unconditional move-to-front on main hits — yielding one regression, one clear winner, and one narrow positive. The winner (v002) reopens the queue-sizing dimension that had been considered settled.

## Patterns

- **Cross-feature interactions can invalidate previously successful axes.** Raising the second-chance threshold (v001) was a profitable direction in isolation, but eager promotion introduced in cycle 0007 now deposits items into main already carrying freq==2. The tighter gate evicts those freshly-promoted hot items on their first pass, turning what was a "stale squatter filter" into an accidental hot-item ejector. Threshold adjustments that worked before a structural change may not transfer across it.
- **The small-queue sizing benefit is monotone past the 10% floor.** Shrinking to ~7% continued to improve resident quality by transferring slots to main; the prior characterisation of 10% as near-optimal was an untested assumption rather than an empirical bound. This axis is not yet exhausted.
- **Unconditional MTF on main hits delivers a real but narrow gain.** Proactive recency refresh prevents drift to the eviction tail, but because evict_main()'s reactive recycling already rescues most hot items on Zipfian workloads, the marginal value is incremental rather than structural.

## Open questions

- Does the small-queue benefit continue below ~7%, or is there a floor where the filter becomes too coarse and cold items begin polluting main in meaningful numbers?
- Given that eager promotion sets the entry frequency for main-queue arrivals, is there a second-chance threshold formulation — perhaps frequency-relative rather than absolute — that captures the original intent without penalising recently-promoted items?

## Continuation

The most actionable finding is that small-queue sizing retains room to run: the next cycle should probe at least one further reduction below ~7% to locate the true floor. The second-chance threshold axis is not closed, but any revisit must reason explicitly about the freq==2 entry distribution created by eager promotion — a naive increment repeats v001's failure. The unconditional MTF result (v003) is a confirmed small positive that can serve as a base feature; it does not need further dedicated probing on this workload.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants continued exploiting the small-queue sizing and main-queue protection axes identified in cycle 0008: further shrinking the small queue, stacking MTF on main hits onto the prior base, and extending the eviction recycle window. Only the further queue reduction landed as a winner; MTF on the older base was a narrower positive that didn't surpass it, and the recycle extension was a small loss.

## Patterns

- **The small-queue sizing benefit continues below ~7% without degrading the cold filter.** Reducing to ~5% still admitted enough cold traffic rejection that main gained higher-quality residents — the monotone trend from cycle 0008 holds at least one step further, and the floor remains unfound.
- **Queue sizing and MTF address orthogonal failure modes.** Cold-admission pressure (mitigated by a smaller small queue) is the larger drag on this workload; positional drift in main (corrected by MTF) is a secondary inefficiency. The two mechanisms combine additively rather than substitutively, but queue sizing dominates.
- **The freq==1 recycle extension is ruled out under eager promotion.** Items that reach the main-queue tail at freq==1 after eager promotion are genuinely cold on Zipfian access; granting them an extra pass delays fresher admissions without improving their own hit probability. The existing freq>1 threshold is already well-matched to the post-promotion frequency distribution.

## Open questions

- Is there a true floor to the small-queue sizing benefit, and does it manifest as a sudden quality cliff when the filter becomes too coarse, or a gradual diminishing return?
- Would combining the ~5% queue size with MTF on main hits yield an additive gain exceeding either mechanism alone, given that they target orthogonal failure modes?

## Continuation

The highest-priority probe for the next cycle is pushing the small queue below ~5% (e.g., capacity/28 or capacity/40) to locate the floor empirically. Simultaneously, combining the winner's queue size with MTF is worth one explicit test — both mechanisms are confirmed positives addressing different failure modes and their interaction hasn't been measured at the ~5% base. The recycle-window axis (freq threshold on evict_main) is closed; do not revisit it.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

This cycle tested three focused hypotheses: stacking MTF on main-queue hits onto the ~5% small-queue base, shrinking the small queue one step further to ~3.6%, and redirecting ghost population from cold evictions to main-queue evictions. The winner combined the ~5% queue with MTF — confirming the additive interaction that cycle 0009 left unmeasured.

## Patterns

- **MTF and queue sizing compound additively, and their combination beats further queue shrinking alone.** Both mechanisms target distinct failure modes on this workload, and stacking them at the ~5% base outperformed the next queue-size reduction step. The two levers have now been jointly confirmed as the new best baseline.
- **The small-queue benefit curve is flattening but has not cliffed.** The ~3.6% step still improved hit rate, but the incremental gain was visibly smaller than the ~5% step, and it was not strong enough to win over the MTF combination. The true floor is close but not yet breached.
- **Ghost signal value is driven by volume, not semantic quality.** Redirecting ghosts to track main-queue evictions instead of cold small-queue rejections collapsed coverage: main evictions are sparse and describe items whose working-set tenure is already exhausted, so almost none of the re-accessing medium-popularity keys got fast readmission. This axis is definitively closed.

## Open questions

- Is the small-queue floor a gradual diminishing return or a sudden quality cliff, and does it fall between ~3.6% and some smaller fraction?
- Would combining MTF with the ~3.6% queue (rather than ~5%) yield a further additive step, or has the positional-drift failure mode already been largely eliminated by the time the smaller queue promotes items?

## Continuation

The new baseline is ~5% small queue plus unconditional MTF on main hits; the next cycle's hypotheses-writer should treat this as the starting point. The most informative remaining probe is one more queue-size reduction step (e.g., capacity/40) combined with MTF, to test whether the additive property holds at a tighter admission filter and to locate the floor empirically. The ghost-source axis and the recycle-window axis are both closed; do not revisit them.


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
