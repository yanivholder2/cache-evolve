# Cross-Cycle Insights

## Recurring wins

- **Aggressive small-queue admission reduction** — Every variant tightening the admission gate (1/4→1/5→1/6→1/8→1/9→1/10 across cycles 0002–0005) delivered consistent hit-rate improvement with no observed degradation or utilization cliff, even at extreme selectivity ratios. This is the single dominant lever across all five cycles.

- **Conservative frequency-depth scaling** — MAX_FREQ 3→4 won decisively in Cycle 0001. Later orthogonal tests (Cycles 0003–0005) confirm frequency-depth is genuinely complementary: modest increases (4→5) deliver measurable but modest gains that compound with structural selectivity (e.g., 1/8 + MAX_FREQ=5) without cancellation.

- **Orthogonal axis compounding** — Structural selectivity and frequency-depth dimensions are provably independent: combined variants (1/6+MAX_FREQ=4, 1/8+MAX_FREQ=5) yielded improvements from both directions simultaneously, validating that tuning one axis does not exhaust the other.

## Recurring failures

- **Ghost-hit frequency protection** — Across three separate cycle-variants (Cycle 0001 v002, 0002 v002, 0003 v002), granting items prior-survival frequency credits (whether upfront or earned on re-admission) consistently failed to improve hit-rate. Pattern is consistent: past survival does not predict future reuse in this workload; frequency protection must be *earned through actual re-access*, not gifted based on history.

- **Frequency-threshold promotion selectivity** — Attempts to tighten the small-to-main promotion frequency threshold (Cycle 0003 v002: freq>1→freq>2) hurt rather than helped performance. Frequency signals are too coarse-grained to support this decision point; size-based admission gating consistently outperforms frequency-based filtering at this scale.

- **Dynamic/adaptive thresholds without structural recalibration** — Cycle 0001 v003's dynamic promotion threshold (congestion-responsive 80% filtering) showed zero measurable impact, suggesting the workload either avoids sustained congestion patterns or the frequency requirement misaligns with its access dynamics. Frequency-based adaptation without simultaneous structural selectivity tuning appears ineffective.

## Open questions

- **Where is the saturation boundary for small-queue reduction?** All variants from 1/5 through 1/10 improved monotonically with no cliff or degradation. Does 1/11 or 1/12 finally degrade, or does the workload tolerate even more extreme selectivity? The absence of a utilization cliff through 1/10 is remarkable and warrants aggressive boundary mapping.

- **Can workload characteristics predict optimal structural parameters a priori?** Reuse-depth histogram, burstiness, or access-pattern skew could potentially predict the saturation boundary for this (and future) workload(s), eliminating incremental search. Five cycles of consistent directional improvement suggest the parameter space is predictable if characterized correctly.

- **Does saturation manifest abruptly or gradually?** If a cliff exists, does hit-rate degrade sharply (forced utilization failure) or gently (frequency signals become too coarse to distinguish reuse at extreme selectivity)? This affects how aggressively future cycles should search.

- **Can adaptive admission policies exceed fixed structural boundaries?** The extreme selectivity gains (1/10+) suggest either exceptional reuse clustering or that fixed-ratio approaches are leaving value on the table. Dynamic per-phase or learned admission selectivity might outperform fixed constants.

## Per-policy notes

- **S3-FIFO frequency calibration** — The baseline frequency-based eviction thresholds (MAX_FREQ=3) are finely tuned but leave headroom: modest depth increases (3→4) deliver clear improvements, while aggressive jumps (4→5 in later cycles) show diminishing returns. The frequency mechanism works best as a "prove reuse through actual access" filter rather than speculative protection.

- **S3-FIFO small-queue role** — Contrary to the small queue's nominal function (rapid churn buffer), this workload rewards treating it as an *admission gate*: structural tightening from 1/4 to 1/10 capacity forces increasingly selective filtering by frequency, with no degradation in working-set coverage. The frequency promotion threshold (freq > 1 for graduation to main) is a surprisingly effective separator of signal from noise even under extreme admission pressure.

- **S3-FIFO orthogonal tuning axes** — Structural selectivity (queue sizing) and frequency-depth (MAX_FREQ) are genuinely independent levers with different leverage: structural selectivity dominates by 2–4× in absolute improvement magnitude, but frequency-depth provides complementary gains without interference. The two-axis optimization space remains incompletely explored at high selectivity ratios (1/8+) paired with higher MAX_FREQ values.
