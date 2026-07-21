# Cross-Cycle Insights

## Recurring wins

- **MRU-end insertion** has been the most durable single change: introduced in Cycle 1, confirmed additive through Cycle 2, and carried as a stable anchor into every subsequent cycle without interaction regressions.
- **Structural overhead removal** (timestamp elimination) delivered a dual dividend — smaller per-item footprint and eliminated syscalls — with no hit-rate or latency penalty, and stacked cleanly with MRU-end insertion.
- **Vacancy-gating promotion fires** (rather than relying on threshold frequency alone) is the single most reliable architectural pattern: it eliminates cascades by construction, yielding hit-rate gains with zero regressions across Cycles 3–5.
- **Additive stacking** has held up repeatedly: MRU-end + timestamp-free (Cycle 2), vacancy-gated promotion + MRU base (Cycle 3), LFU-min eviction + vacancy-gated promotion (Cycle 5). Independent mechanisms on orthogonal subsystems compound without interference.
- **LFU-minimum eviction selection** was independently validated in Cycle 5 as a distinct, stackable lever — hit-rate improves over LRU positional selection even without any promotion gating, confirming the eviction side has genuine headroom.
- **Symmetric vacancy-gated probing re-entry** (Cycle 5 v002) added a small regression-free hit-rate lift, confirming the vacancy-guard pattern transfers safely to the probing path.

## Recurring failures

- **Threshold-only frequency gating (no vacancy guard)** consistently produces cascade-driven tail-latency regressions regardless of threshold value: at ≥2 (Cycle 2), ≥3 (Cycle 3), and again as a comparison point in Cycle 4. The root cause is architectural — a gate that can fire into a full tier will always cascade; threshold tuning only adjusts the rate of harm, not its existence.
- **Stash-bypass on the MRU-end base** reversed the gain it had on the LRU base. MRU-end insertion makes target saturation the common case, so bypass fires far more often, converting the probing tier into a spillover sink. Workload-base interaction invalidated the prior win.
- **Rare joint-condition mechanisms** consistently fail to produce a detectable signal on this workload: target-full + probing-vacant at stash-hit (Cycle 4), and LFU counter reset for stash re-entrants (Cycle 4) both fired too infrequently for measurement at this cache size and trace length.
- **Overlay coverage gaps** caused silent structural regressions in Cycle 4 when child hypotheses only modified a subset of the files changed by the parent, reverting the parent's struct compaction. The regression attributed to the hypothesis was actually an omission.
- **Raising the vacancy-gated threshold beyond ≥2** (to ≥3) on the clean vacancy-gated base produced a fractional hit-rate loss, indicating ≥2 is near the local optimum for this workload's reuse distribution. Further threshold tightening appears counterproductive.

## Open questions

- Can the O(n) LFU-minimum eviction scan be replaced with an O(1) maintained index (per-bucket min-counter slot or a small frequency-sorted auxiliary) to capture the hit-rate gain without the consistent put p99 regression?
- Are there other struct fields or per-operation syscalls in the baseline that carry dead-weight similar to the eliminated timestamp — the timestamp discovery suggests the baseline has not been fully audited?
- Can the vacancy gate and threshold ≥3 be combined (fire only when counter ≥3 AND probing has space) to test whether precision and safety together push hit-rate higher than either alone?
- Is stash-bypass salvageable on the MRU-end base if guarded by a probing-vacancy check analogous to the vacancy-gate fix, converting an unguarded bypass into a structurally safe one?
- What is the combined latency footprint of the full three-way stack (LFU-min eviction + vacancy-gated target promotion + vacancy-gated probing re-entry) once the O(n) scan is the only remaining regressor?
- At what cache-to-working-set ratio do rare joint conditions (target-full + probing-vacant simultaneously) fire frequently enough to become measurable, making those mechanisms worth revisiting?
- Is there a mechanism to verify overlay coverage completeness at hypothesis-generation time, preventing silent omission-driven regressions from contaminating future measurements?

## Per-policy notes

- **LRU positional eviction (baseline):** benefits most from insertion-position changes (MRU-end) and structural overhead removal; hits a headroom ceiling when only promotion tuning is applied at the current workload ratio.
- **Vacancy-gated frequency promotion (≥2 threshold):** the cleanest, most durable policy-level win — zero cascades by construction, additive with eviction changes, and ≥2 is empirically near-optimal for this trace (≥3 loses hit-rate, Cycle 4).
- **LFU-minimum eviction selection:** independently improves hit-rate over LRU baseline (Cycle 5 v003) and stacks additively with vacancy-gated promotion (Cycle 5 v001); the O(n) bucket scan reliably inflates put p99 in all tested configurations — resolving this cost is the primary blocker for wider adoption.
- **Forced LFU eviction policy override (Cycle 1 v002):** rejected by simulator timeout, not by performance — merit remains unknown and unresolved after five cycles.
- **Threshold-only frequency gate (no vacancy guard):** tested at ≥2 and ≥3 across Cycles 2–4; consistently produces tail-latency regressions; deprecated as a standalone policy, only viable when paired with a vacancy constraint.
- **Vacancy-gated probing re-entry:** thin but regression-free gain on this workload; likely to contribute more at higher probing residency or smaller cache-to-working-set ratios where demoted-but-reused items accumulate at higher rates.
