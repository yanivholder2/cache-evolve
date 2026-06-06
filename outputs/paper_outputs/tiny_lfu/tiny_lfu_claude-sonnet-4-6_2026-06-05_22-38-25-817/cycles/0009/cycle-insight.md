# Cycle 0009 Insight

## Cohort outcome

All three variants revisited policy axes previously closed under a noisy sketch, now re-evaluated against the fully-accurate 8×-width + conservative-update baseline. Only the PROMO_THRESHOLD reduction (v001) was selected winner, producing a small but real gain; both the wider probationary victim scan (v002) and frequency-ordered protected demotion (v003) registered directionally positive but negligible movements and were not selected.

## Patterns

- **The cycle-0008 directive was correct but modest in yield.** Recalibrating policy knobs under sharper sketch signal did unlock a real gain from PROMO_THRESHOLD — confirming that the gate was miscalibrated against the old noisy baseline — but the frequency band unlocked (freq 6–7) is a thin slice of the Zipfian access distribution, so the gain is marginal rather than structural.
- **All three variants were positive; none regressed.** This is the first cycle in which every variant moved in the right direction. It reflects that the remaining policy space has no obviously wrong directions left — only low-leverage ones — which itself signals proximity to a ceiling.
- **Probationary victim-scan width and protected demotion scan are low-leverage, not broken.** Both v002 and v003 confirmed that extending the respective scan windows under accurate estimates is sound and directionally correct, but the protected segment's tight PROMO_THRESHOLD entry bar leaves genuine frequency spread narrow, and the LRU tail of probationary is already well-characterized by k=4. Neither axis offers further meaningful tuning room.
- **PROMO_THRESHOLD is still the highest-leverage policy knob.** Even at its new value of 6 the gain is marginal; further lowering may exist but is unlikely to compound substantially given the thinning frequency bands below 6.

## Open questions

- Does PROMO_THRESHOLD=5 (or lower) unlock another thin slice, and at what threshold does protected-segment churn begin to outweigh the admission benefit?
- Is the remaining hit-rate gap now attributable to irreducible workload entropy rather than any policy choice — i.e., has the design genuinely reached its ceiling for this trace?
- Would the SLRU segment-size split (currently 90/10 probationary/protected) yield more leverage now that both admission and demotion policy are well-calibrated?

## Continuation

Adopt v001's PROMO_THRESHOLD=6 as the new baseline. The probationary scan width and protected demotion scan are closed as low-leverage. The two remaining untested structural axes are the SLRU split ratio and further PROMO_THRESHOLD reduction; of these, the split ratio is higher-risk but potentially higher-reward if the protected segment is undersized relative to the hot-item working set. If the split also disappoints, the design has reached its ceiling for this workload and trace, and further tuning investment is unlikely to pay off.
