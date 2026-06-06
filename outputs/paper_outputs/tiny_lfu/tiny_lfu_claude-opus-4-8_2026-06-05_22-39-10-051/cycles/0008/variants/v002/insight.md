# Variant v002 Insight

## Helped

- hit-rate: Enlarging the protected SLRU tier to 0.9 of capacity (`protected_cap = (capacity * 9) / 10`, 1843 of 2048 slots) shielded marginally more frequency-proven keys from probationary eviction, producing a barely-perceptible uptick over the parent's 0.8 sizing.

## Hurt

- (none observed): Halving probation from 410 to 205 slots cost nothing measurable; the always-present victim invariant held, and the smaller probationary region did not starve admission of churn-survival room enough to drop a hit.

## Takeaway

Pushing the protected fraction above 0.8 for the first time confirms the cycle-0005 direction (larger protected tier is favored) but reveals the curve is essentially flat at the top: at 0.9 the gain is within noise of the parent. The protected tier was not meaningfully under-sized at 0.8 — on workload-a the keys saved by the extra 205 protected slots are nearly the same keys probation would have retained anyway, so this is the search reaching the diminishing-returns plateau of SLRU sizing rather than an exploitable lift.
