# Variant v001 Insight

## Helped

- hit-rate: Expanding the protected segment from 80 % to 90 % allows more of the Zipfian hot set to reside stably in protected, reducing the rate at which frequently-accessed items get demoted back to probationary and then re-compete for re-promotion.

## Hurt

## Takeaway

Growing the protected segment to 90 % moves hit rate in the right direction on this Zipfian trace, but the gain is negligible — the hot set was already largely captured at 80 %, so the extra headroom covers only a thin margin of borderline-hot items. Halving the probationary buffer to 10 % caused no visible harm, confirming that the TinyLFU admission gate is the primary filter for cold items and the probationary segment is acting mainly as a pass-through staging area rather than a meaningful contention zone. Further SLRU split tuning is unlikely to yield large returns; more significant levers are probably in the admission or frequency-decay logic.
