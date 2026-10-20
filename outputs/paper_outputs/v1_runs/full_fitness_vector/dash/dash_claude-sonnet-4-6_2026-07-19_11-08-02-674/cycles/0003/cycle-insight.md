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
