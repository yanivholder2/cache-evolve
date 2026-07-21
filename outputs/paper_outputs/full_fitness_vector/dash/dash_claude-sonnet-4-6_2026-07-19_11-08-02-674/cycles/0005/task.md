# Task: cycle-insight-writer

Cycle: 0005
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0005

## Winners

- v001
- v002
- v003

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0005\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0005\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0005\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0005\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0005\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0005\hypotheses\H03.md

## Prior cycles digest

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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle stress-tested the MRU-end + timestamp-free base by layering three independent ideas on top of it: stash saturation bypass, a recalibrated frequency gate, and a vacancy-gated frequency promotion. The clearest win was structural — vacancy-gating gate-fires to open probing slots eliminated cascade evictions entirely and delivered a regression-free hit-rate lift. The threshold-tuning approach improved on the prior cycle's calibration but could not fully suppress tail-latency inflation. The stash-bypass, which had succeeded on the LRU-end base, reversed on the MRU-end base, revealing a previously hidden interaction between insertion position and bypass-fire rate.

## Patterns

- **Structural cascade elimination beats threshold tuning.** Vacancy-gating promotion fires so that no cascade can occur (v003) produced a clean two-dimensional outcome, while raising the frequency threshold to ≥ 3 (v002) only softened the latency regression without curing it. The root problem is architectural — a gate that fires into a full tier will always trigger a cascade regardless of how rarely it fires.
- **MRU-end insertion changes tier pressure dynamics across the board.** Stash-bypass was a hit-rate win on the LRU-end base but a regression on the MRU-end base because back-insertion makes target saturation the common case, so the bypass fires far more often, converting the probing tier into a spillover sink for stash traffic.
- **All three strategies can be ordered by cascade risk.** From safest to most aggressive: vacancy gate (zero cascades by construction) → threshold ≥ 3 (cascades suppressed by rate) → stash-bypass on MRU base (cascades amplified by insertion position). The hit-rate benefit and the latency cost track this ordering.

## Open questions

- Can the vacancy gate and the higher threshold be combined — gate fires only when probing has space AND counter ≥ 3 — to see whether precision and safety together push hit-rate higher than either alone?
- Is stash-bypass salvageable on the MRU-end base if guarded by a probing-vacancy check, analogous to what v003 did for frequency promotion?
- Does the vacancy gate's marginal hit-rate lift grow at higher cache occupancy (smaller cache relative to working set), where probing vacancies are rarer and gate-fires are naturally throttled more aggressively?

## Continuation

Next cycle should use v003's vacancy-gated promotion as the new starting point — it is the only variant this cycle that improved hit-rate with no regressions. The highest-value experiment is combining the vacancy gate with a threshold of ≥ 3 to determine whether restricting gate-fires to items with stronger reuse evidence amplifies the hit-rate gain. The stash-bypass direction should be retried, but gated on probing vacancy in addition to target fullness, mirroring the structural fix that made v003 succeed where v002 fell short. The unguarded frequency-gate idea (threshold-only calibration) appears to have a fundamental latency ceiling and should be deprioritized unless combined with the vacancy constraint.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle attempted three refinements on top of v003's vacancy-gated frequency promotion: a stricter ≥ 3 frequency threshold, a vacancy-gated stash bypass, and an lfu_counter reset at stash-to-target promotion. No variant moved any measured dimension positively — every regression was either structural (overlay coverage gap silently dropping the parent's Item-struct compaction) or workload-dependent (the target scenarios simply do not occur often enough on this trace).

## Patterns

- **Overlay coverage gaps are a first-class failure mode.** Two of three variants touched only `dash_segment.rs` and silently reverted the parent's `item.rs` struct reduction, re-inflating per-item footprint and worsening tail latency. The regression did not come from the hypothesis under test — it came from incomplete inheritance of the parent's accumulated changes. Any multi-file parent change must be fully re-applied in descendant overlays, or the measurement conflates intent with omission.
- **The vacancy gate's ≥ 2 threshold appears close to optimal for this workload.** Raising to ≥ 3 produced a fractional hit-rate loss rather than a gain, indicating that a real cohort of genuinely reusable items sits between the two thresholds. The prior cycle's threshold-only (no vacancy guard) ≥ 3 result had a different base, so the comparison was confounded; on the clean vacancy-gated base, stricter filtering is net harmful.
- **The joint stash-bypass condition fires too rarely to be measurable.** Requiring both target-full and probing-vacant simultaneously at stash-hit time is apparently an uncommon joint event on this trace, making the bypass mechanism inert. The hypothesis was structurally sound but the workload does not generate the precondition frequently enough.
- **Counter-carrying oscillation is not a live pattern in this workload.** The lfu_counter reset was designed to eliminate premature re-promotion of stash-re-entrants with stale counts, but no hit-rate signal emerged, implying the oscillation scenario is too rare on workload-a to be detectable at this cache size.

## Open questions

- Is there a mechanism to verify at hypothesis-generation time that a child overlay carries forward all files modified by its parent, preventing silent structural regressions from coverage gaps?
- At what cache-to-working-set ratio does stash traffic increase enough that the target-full + probing-vacant joint condition fires at a detectable rate?
- If the ≥ 2 threshold is near-optimal, are there other promotion axes (e.g., recency combined with frequency, or a decay on the counter over time) that could improve selectivity without the crude threshold knob?

## Continuation

Next cycle should not further tune the frequency threshold — the ≥ 2 vacancy-gated gate from Cycle 3's v003 appears to be the local optimum for this workload. The overlay-coverage-gap finding is the most actionable output of this cycle: future hypotheses that descend from multi-file parents must explicitly enumerate all inherited file changes, not just the files the new hypothesis modifies. Rather than continuing to refine promotion sensitivity, the next cycle should explore an orthogonal axis — for example, eviction ordering within a tier (which item gets displaced when a full bucket must eject), or the promotion path for items accessed in the probing tier rather than the target tier. The vacancy-gate and MRU-end insertion are now stable anchors; the remaining headroom is likely in eviction selection rather than promotion gating.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
