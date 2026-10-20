# Variant v003 Insight

## Helped

- (none): No dimension improved; scaling the ghost capacity to 1.5x the parent's value did not yield any gain.

## Hurt

- hit-rate: A longer ghost queue lets staler evicted keys qualify for direct insertion into main on a ghost hit, so the admission filter loosens slightly and admits marginally more one-time-reuse objects. The loss is very small.

## Takeaway

The parent's shortened ghost length is already at or very near the optimum for this workload. Going 1.5x longer in the direction earlier notes called harmful gives only a negligible loss, so ghost length has a flat, low-sensitivity response here. Further tuning of this constant is unlikely to pay off, and effort should move to other mechanisms such as small-queue sizing or the promotion rule. Caveat: I did not inspect the overlay source, so this reading rests on the hypothesis text and the fitness vectors.
