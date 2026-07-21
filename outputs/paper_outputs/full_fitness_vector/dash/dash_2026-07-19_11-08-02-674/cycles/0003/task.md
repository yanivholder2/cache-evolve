# Task: cycle-insight-writer

Cycle: 0003
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0003

## Winners

- v002
- v003
- v001

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0003\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0003\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0003\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0003\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0003\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0003\hypotheses\H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

This cycle explored three independent axes — promotion-insertion ordering, forced eviction-policy override, and per-item metadata pruning. Both survivors were clean wins with no regressions: one accelerating the warm-up path through the tier hierarchy, the other eliminating structural overhead from every cached item.

## Patterns

- **Orthogonal wins compound cleanly.** The two winners touch entirely separate subsystems (promotion mechanics vs. item struct layout) with no shared code paths, making them strong candidates for stacking in the next cycle.
- **Insertion position had outsized impact.** A single argument flip from LRU-end to MRU-end insertion improved both hit rate and per-op time simultaneously, revealing that the baseline's placement choice was actively working against the tier hierarchy's warm-up logic.
- **Syscall and struct costs are low-hanging fruit.** The timestamp removal produced a dual dividend — smaller memory footprint and eliminated syscalls — with zero hit-rate penalty, suggesting the baseline carries more dead-weight overhead worth auditing.
- **Operational timeout masked the LFU hypothesis.** v002 was rejected by simulator timeout rather than by performance, so the merit of forcing LFU eviction remains genuinely unknown.

## Open questions

- Are there other struct fields or per-operation syscalls in the baseline that provide no eviction advantage on this workload, similar to `timestamp`?
- Is the LFU hypothesis worth retesting with a lighter trace or smaller cache size to get a fitness signal before committing to a full run?
- What happens when MRU-end insertion and timestamp removal are combined — do they interact, or does the improvement simply add?

## Continuation

Next cycle should stack v001 and v003 together as a combined baseline to test whether their independently measured gains are additive. In parallel, the LFU eviction hypothesis from v002 deserves a retry under a configuration that avoids timeout — either a faster trace or a reduced policy surface — before the idea is discarded. Further metadata audits similar to the timestamp discovery are likely to yield more free wins.


### Cycle 0002

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
