# Variant v002 Insight

## Helped

None. The two 131072 cells are identical to the parent's, as the hypothesis predicted. `probation_len(1024)` is still `1024 / 4 = 256`, which is already above the new 96 floor, so large buckets are configured exactly as in cycle 0005 v003.

## Hurt

- hit-rate (hm_0 @ 16384): With probation at 96 slots, a 128-slot bucket keeps only 32 main slots, down from 64. That frequency-ranked main region is now too small to hold hm_0's hot set. Promoted blocks start at counter 0, so they are the first victims of the next sweep and get pushed out by later promotions. This is the drop the hypothesis's refutation clause warned about, and it is the larger of the two losses.
- hit-rate (web_0 @ 16384): web_0 also lost a little, so the deeper ring did not pay off even on the trace it was aimed at. Any extra late second touches caught in the 96-slot ring were outweighed by losses in the smaller main region. There, a promoted block competes with only 31 others and is evicted before its own reuse comes back.

## Takeaway

Both refutation conditions in H02 held: web_0 did not beat the parent, and hm_0 lost more than web_0 gained (web_0 actually lost too). For GCLOCK+probation, the optimum for 128-slot buckets lies between a 64- and a 96-slot ring. Probation depth and main capacity become a zero-sum trade somewhere past a 50/50 split. The parent's view that depth and main capacity were not in competition held only up to half the bucket. Further gains at 16384 should come from levers other than depth, such as a non-zero counter on promotion or ghost/adaptive sizing, with the floor kept at 64 or between 64 and 96. Per-op-time was not measured, so the cheaper 32-slot main sweep the hypothesis expected is unconfirmed.
