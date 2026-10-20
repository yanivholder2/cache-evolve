# Variant v002 Insight

## Helped

## Hurt

## Takeaway

This variant was meant to skip the reuse-counter store on hits to already-saturated entries, a change designed to be neutral on hit-rate and to lower per-op-time. The only recorded dimension is hit-rate, and it is unchanged from the parent, which is consistent with the change being behaviour-preserving. Per-op-time is not in the fitness vector, so this variant's cost effect is unmeasured and no speedup can be credited. Caveat: the overlay source could not be read in this pass, so I could not confirm that the write skip was implemented as the hypothesis describes.
