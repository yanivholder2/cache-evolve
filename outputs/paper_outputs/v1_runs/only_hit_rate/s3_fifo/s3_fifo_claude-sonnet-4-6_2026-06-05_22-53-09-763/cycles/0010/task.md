# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0010

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
