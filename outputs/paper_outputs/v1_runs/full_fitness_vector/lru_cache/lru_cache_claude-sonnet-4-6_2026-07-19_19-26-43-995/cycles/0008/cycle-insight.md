# Cycle 0008 Insight

## Cohort outcome

This cycle probed the final steps of quota compression (capacity/32), a frequency-guided 2-candidate eviction scheme, and a cross-parent validation of the 93.75% quota on the link-array structural baseline. No variants won; the most significant result is a verdict reversal: the per-op-time cost of the high protected quota is real, not a measurement artifact.

## Patterns

- **The per-op-time regression is quota-induced, not measurement noise.** v003 applied the same 93.75% quota change to a confirmed clean-floor parent (the link-array baseline) and reproduced both the hit-rate gain and the latency penalty. This retroactively recharacterizes prior cycles' "winners" at capacity/16: they carry a genuine per-op-time trade-off that was obscured by anomalous zero-floor baselines, not escaped by parent selection.
- **Quota compression is in steep diminishing returns on hit-rate.** v001's step from capacity/16 to capacity/32 delivered a visibly smaller improvement than each prior halving, confirming the trend flagged in Cycle 0007 is steepening toward a plateau or reversal.
- **Frequency-guided eviction requires probationary runway it cannot have at high quotas.** v002 closed the last open branch from Cycle 0007: at capacity/16 probationary size, items are evicted before the flat frequency table accumulates distinguishable signal, making the 2-candidate comparison a coin flip. Frequency hints and deep-quota operation are structurally incompatible under this workload's access pattern.
- **The 93.75% quota now has a confirmed cost on every tested parent.** The combination of high hit-rate and zero latency penalty has not been achieved at this quota level; both signals cannot be won simultaneously with the current policy structure.

## Open questions

- Is there a quota level — somewhere between the current baseline (87.5%) and the best hit-rate setting (93.75%) — where the LRU boundary enforcement overhead remains below the measurement floor while still capturing a fraction of the hit-rate gain?
- The latency cost is linked to increased overflow frequency (demotion from protected to probationary). Could batching or amortizing demotion events reduce the per-op penalty without changing the effective quota?
- With quota compression, admission filtering, and frequency-guided eviction all exhausted, what structural or algorithmic axes remain unexplored on this workload?

## Continuation

The next cycle should treat the 87.5% quota (capacity/8 probationary) as the confirmed clean baseline — it is the last setting known to be cost-free on both dimensions — and accept that moving to 93.75% carries a real latency penalty. Two directions worth testing: (1) probe an intermediate quota (e.g., capacity/12 or capacity/24) to determine whether a smaller overflow rate stays below the latency measurement floor while recovering part of the hit-rate gain; (2) pivot away from quota and explore a structurally different axis such as reducing hash-map lookup overhead or node memory layout changes that could improve per-op-time independently of policy. Do not revisit admission filters, frequency sketches, or demotion ordering — all are conclusively closed. Do not push quota beyond capacity/16 without first resolving the latency trade-off at that level.
