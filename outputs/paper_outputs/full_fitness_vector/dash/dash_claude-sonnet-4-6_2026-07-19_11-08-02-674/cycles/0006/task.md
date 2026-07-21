# Task: cycle-insight-writer

Cycle: 0006
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0006

## Winners

- v001
- v003
- v002

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0006\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0006\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0006\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0006\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0006\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\dash_2026-07-19_11-08-02-674\cycles\0006\hypotheses\H03.md

## Prior cycles digest

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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle explored eviction-side improvements — replacing positional LRU victim selection with frequency-minimum selection — alongside a symmetric application of the vacancy gate to the probing re-entry path. All three variants won: the two LFU-minimum eviction variants improved hit-rate at the cost of per-op tail latency, while the symmetric probing re-entry variant added a small regression-free hit-rate lift.

## Patterns

- **LFU-minimum eviction is an independently validated lever.** v003, which applied frequency-minimum victim selection without any inter-tier promotion gate, confirmed that eviction selection alone improves hit-rate meaningfully over the baseline — isolating it as a distinct mechanism rather than an auxiliary to promotion gating. v001 then showed it also stacks additively on top of the vacancy-gated promotion base.
- **The O(n) eviction scan's latency cost is consistent and non-trivial.** Both LFU-minimum variants (v001 and v003) inflated put p99 regardless of what else was present, confirming the scan overhead is the dominant latency regression source this cycle — not promotion cascades or insertion-position interactions.
- **Symmetric vacancy-gated probing re-entry is safe but thin.** Applying the same vacancy gate to the probing-hit branch (v002) added a fractional hit-rate gain with zero regressions, confirming the pattern is structurally sound. The joint condition (counter threshold met AND target has space) fires infrequently enough on this workload that the gain remains incremental rather than transformative.
- **Eviction selection and promotion gating appear complementary, not redundant.** The ordering of hit-rate lifts — baseline < eviction-only (v003) < eviction + promotion gate (v001) — suggests the two mechanisms protect different item populations.

## Open questions

- Can the LFU-minimum scan's latency cost be eliminated or amortized — for example, by maintaining a per-bucket minimum-counter index or a small frequency-sorted auxiliary structure — so the hit-rate gain can be captured without the tail-latency regression?
- Does stacking all three mechanisms (LFU-min eviction + vacancy-gated target promotion + vacancy-gated probing re-entry) yield additive hit-rate gains, and what is the combined latency footprint once the scan is the only regressor?
- Would the probing re-entry path contribute more on a workload with higher probing residency or a smaller cache-to-working-set ratio, where demoted-but-reused items accumulate in probing at higher rates?

## Continuation

Next cycle's highest-priority direction is replacing the O(n) eviction scan with an O(1) or near-O(1) alternative — a maintained per-bucket minimum-counter slot or a small frequency-ordered auxiliary — to preserve the confirmed hit-rate gain without the tail-latency cost. Until the scan overhead is resolved, stacking further mechanisms on top obscures their individual contributions. v002's probing re-entry is regression-free and should be carried forward in any multi-mechanism stack. The full three-way combination (LFU-min eviction + both vacancy gates) is the natural convergence target once the scan latency problem is solved.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
