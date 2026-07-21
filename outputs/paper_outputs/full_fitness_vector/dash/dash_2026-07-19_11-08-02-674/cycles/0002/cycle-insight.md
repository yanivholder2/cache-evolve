# Cycle 0002 Insight

## Cohort outcome

This cycle pursued three directions simultaneously: confirming that Cycle 1's two winners stack cleanly, probing frequency-based early promotion, and bypassing a saturated target tier on stash hits. All three variants won, making this an unusually productive cohort — though the two new ideas each extracted hit-rate gains at a tail-latency cost absent from Cycle 1.

## Patterns

- **Stacking was confirmed additive.** The combined MRU-end insertion + timestamp removal variant preserved the prior hit-rate gain and reduced struct size by exactly the expected width, with no interaction between the two subsystems. The stacking hypothesis was fully vindicated.
- **Cycle 2's new wins trade hit-rate for p99 latency.** Both the frequency gate and the stash-bypass variants improved hit-rate but raised tail latency (p99 get, p50 put), in contrast to Cycle 1's winners which had no regressions. The shared mechanism is cascade evictions triggered by conditional promotion logic — each gate fires often enough to generate a steady stream of downstream demotions.
- **The frequency gate has the right idea but the wrong threshold.** A threshold of ≥ 2 fires aggressively enough that cascade churn negates much of the hit-rate value; the signal is real (the workload does contain re-accessed non-hottest items) but the sensitivity is too high.
- **Stash-bypass is the cleaner hit-rate win.** Skipping a saturated target on stash hits removes one unnecessary eviction per stash-hit and has a clear rationale; its p99 regression is narrower in scope than the frequency gate's broader per-op overhead, suggesting it has a better hit-rate-to-latency ratio.

## Open questions

- Can the frequency gate's threshold be raised (≥ 3 or ≥ 4) or made fullness-conditional to preserve the hit-rate signal while reducing cascade frequency?
- Do the stash-bypass and frequency gate improve overlapping or complementary item populations — and can they coexist without amplifying each other's cascade costs?
- Are the p99 regressions from both new variants workload-size-dependent, or would they disappear at a different cache-to-working-set ratio?

## Continuation

Next cycle should treat v001 (the stacked Cycle 1 baseline) as the new starting point and layer v003's stash-bypass on top of it, since stash-bypass is the higher-quality hit-rate win and has not yet been tested on the combined baseline. In parallel, the frequency gate from v002 deserves a retry at a higher threshold (≥ 3 or guarded by target fullness) to determine whether the idea is viable once the threshold is calibrated. Combining all three mechanisms — stacking, stash-bypass, and a tuned frequency gate — is the natural end-state to aim for, but the components should be validated pairwise first to detect interactions before committing to a three-way stack.
