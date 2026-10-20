# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0009

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-05_22-53-09-763/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
