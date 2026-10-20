# Variant v003 Insight

## Helped

- hit-rate: The doorkeeper keeps first-sighting keys out of the count-min sketch, which is the mechanism the hypothesis predicted would help the admission gate; on lirs workload-a the effect is negligible and within what a single trace can distinguish from noise.

## Hurt

- metadata-size: The doorkeeper adds handle state to `TinyLfuCache` (its struct footprint grew), and the bit-vector storage the hypothesis sized at about 8 KiB does not appear in the reported metadata breakdown, so the memory cost is under-reported.
- per-op-time: The put path now consults the doorkeeper before touching the sketch and clears it on each aging step, which is the most likely source of the worse put tail latency.

## Takeaway

Gating frequency counts behind a one-hit doorkeeper barely changes hit rate on this workload, so the admission gate's behaviour is not limited by one-hit-wonder pollution of the sketch. The change mainly adds bookkeeping cost, and any follow-up should either measure the doorkeeper's heap bytes explicitly or test on a trace where one-hit keys dominate.
