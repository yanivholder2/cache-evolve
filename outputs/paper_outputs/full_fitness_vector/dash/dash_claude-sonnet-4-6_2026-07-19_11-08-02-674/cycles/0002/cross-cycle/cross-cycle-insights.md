# Cross-Cycle Insights

## Recurring wins

- **MRU-end insertion** has been the single highest-leverage change so far: realigning insertion position with the tier hierarchy's warm-up direction improved both hit rate and per-op time simultaneously, with no regressions.
- **Struct/metadata pruning** (timestamp removal) consistently delivers a dual dividend — smaller per-item footprint and eliminated syscalls — at zero hit-rate cost, suggesting the baseline carries dead-weight overhead worth continuing to audit.
- **Independent wins stack cleanly.** When two winners touch entirely separate subsystems (promotion mechanics vs. item layout), their gains have proven fully additive with no cross-subsystem interaction. Stacking is a safe strategy as long as the subsystems remain orthogonal.

## Recurring failures

- **Aggressive conditional-promotion thresholds generate cascade churn.** Every variant that fires a conditional promotion gate at a low threshold (frequency ≥ 2, saturation-bypass on most hits) has raised p99 and p50 latency by triggering a steady stream of downstream demotions. The signal from these ideas is real, but sensitivity consistently overshoots — threshold calibration is the unsolved sub-problem.
- **The LFU forced-eviction hypothesis has produced no fitness signal yet.** It was eliminated by simulator timeout rather than performance, so it is neither confirmed nor refuted. Retesting under a lighter configuration remains warranted before the idea is discarded.

## Open questions

- Can the frequency gate's threshold be raised (≥ 3 or ≥ 4), or made conditional on target-tier fullness, to preserve the hit-rate signal while reducing cascade frequency?
- Are there additional struct fields or per-operation syscalls in the baseline that carry no eviction advantage on this workload, analogous to the timestamp discovery?
- Do stash-bypass and the frequency gate act on overlapping or complementary item populations — can they coexist without amplifying each other's cascade costs?
- Are the p99 regressions from the Cycle 2 conditional-promotion variants workload-size-dependent, or do they persist across different cache-to-working-set ratios?
- Is the LFU hypothesis testable under a reduced trace or smaller cache size before committing to a full simulation run?

## Per-policy notes

- **LRU (baseline):** MRU-end over LRU-end insertion is a dominant win; the baseline placement was actively antagonistic to tier warm-up. No regressions have appeared from this change across either cycle.
- **LFU (forced override):** Has not completed a simulation run (timed out in Cycle 1); eviction quality on this workload is genuinely unknown. Retry at reduced scope before closing this thread.
- **Frequency-gated promotion:** The workload does contain re-accessed non-hottest items, so the promotion signal is real. However, threshold ≥ 2 fires too broadly and the cascade cost exceeds the hit-rate gain. Requires calibration; not a dead end.
- **Stash-bypass (saturation-aware routing):** Skipping a saturated target tier on stash hits removes one unnecessary eviction per event and shows a better hit-rate-to-latency ratio than the frequency gate. It is the highest-quality unvalidated hit-rate lever and the strongest candidate for the next stacking experiment on the combined baseline.
