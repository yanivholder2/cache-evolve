# Cross-Cycle Insights

## Recurring wins

- **Structural cascade elimination beats threshold tuning.** Across cycles 2–3, every attempt to suppress cascade evictions via rate-limiting (threshold tuning) left a residual latency regression, while architectural gating (vacancy check before firing) produced clean, regression-free hit-rate gains. The pattern is reliable enough to treat as a design principle.
- **Stacking orthogonal wins is additive.** Cycle 2 confirmed that MRU-end insertion and timestamp removal — which touch entirely separate subsystems — compose without interaction. Combining verified independent wins before adding new ideas has been a consistently safe strategy.
- **Struct/metadata audits yield free wins.** Timestamp removal delivered smaller memory footprint and eliminated syscalls with zero hit-rate cost. The baseline appears to carry further dead-weight overhead that has not yet been audited.
- **MRU-end insertion is a durable foundation.** The insertion-position flip has been the starting point for every winning variant since cycle 1 and has not been dethroned by any subsequent change.

## Recurring failures

- **Threshold-only calibration cannot cure cascade latency.** Raising the frequency gate from ≥ 2 to ≥ 3 narrowed but did not eliminate the p99 regression. A gate that can fire into a full tier will always cause cascades regardless of firing frequency; tuning the rate is treating a symptom, not the cause.
- **Stash-bypass without a vacancy guard reverses on the MRU-end base.** The bypass won on the LRU-end base (cycle 2) but regressed on the MRU-end base (cycle 3) because back-insertion makes target saturation the common case, dramatically increasing bypass-fire rate. The mechanism is not portable across insertion-position regimes without a structural guard.
- **LFU forced-eviction remains untested at meaningful scale.** The hypothesis has been discarded twice — once by simulator timeout, once by deprioritization — without ever producing a fitness signal. It cannot be confirmed or ruled out yet.

## Open questions

- Can the vacancy gate and a threshold of ≥ 3 be combined to restrict promotion fires to items with both open probing slots and demonstrated reuse, pushing hit-rate higher than either alone?
- Is stash-bypass salvageable on the MRU-end base if gated on probing vacancy, mirroring the structural fix that made vacancy-gated frequency promotion succeed?
- Are there additional struct fields or per-operation costs in the baseline — analogous to `timestamp` — that carry no eviction advantage on this workload?
- Does the vacancy gate's hit-rate lift grow at higher cache occupancy, where probing vacancies are rarer and gate-fires are naturally throttled further?
- Is the LFU eviction hypothesis worth testing on a lighter trace or smaller cache to finally get a fitness signal before the idea is permanently set aside?

## Per-policy notes

- **LRU / MRU-end insertion:** The single highest-leverage change across all cycles. Flipping insertion from LRU-end to MRU-end simultaneously improved hit rate and per-op time by aligning placement with the tier hierarchy's warm-up logic. Every subsequent winner has built on this.
- **Frequency-based promotion (frequency gate):** The underlying signal is real — the workload contains re-accessed non-hottest items — but the mechanism is sensitive to tier fullness. At threshold ≥ 2 the cascade cost dominates; at ≥ 3 it is softened but not eliminated. Vacancy-gating is the structurally sound path forward for this policy.
- **Stash-bypass:** A clean hit-rate win under LRU-end insertion; becomes a regression under MRU-end insertion due to amplified bypass-fire rate. Should be treated as insertion-position-dependent and retried only with a vacancy guard.
- **LFU forced eviction:** Timed out in cycle 1, never retried. No fitness signal available; eviction merit is genuinely unknown.
