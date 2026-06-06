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
