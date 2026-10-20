# Variant v003 Insight

## Helped

- per-op-time: The single-hash, bit-shift row derivation and the early return on a saturated counter skip the multi-row conservative-update pass, so both get and put medians dropped. The put tail (p99) improved most, which fits the sketch work being a large part of the put path. The get p99 stayed flat, so the gain is in the typical case and not the worst case.

## Hurt

- hit-rate: Nothing meaningful. The admission rule and decay policy were left alone, so hit-rate stayed essentially level. The small upward drift is within what changed index derivation (different collision patterns) could produce by chance.
- metadata-size: Nothing. The sketch layout and node size are unchanged.

## Takeaway

Making the sketch increment cheaper (one hash, shift-derived indices, early exit at saturation) is close to a free speed win for this cache, because it doesn't touch the admission logic that decides hit-rate. This was written without access to the overlay source diff, so the interpretation rests on the hypothesis text and the fitness vectors. The data can't separate the saturation early-exit from the cheaper hashing, so a follow-up should test them separately.
