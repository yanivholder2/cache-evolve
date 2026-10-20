# Variant v001 Insight

## Helped

## Hurt

- hit-rate: The hypothesis folds the saturating reuse counter into the one-bit seen-in-stash marker and deletes the saturated-counter gate and the store-skip guard, so reuse depth no longer informs promotion or store decisions. That loss of information is the most likely source of the small hit-rate drop, which sits just outside the tolerance the hypothesis set.

## Takeaway

Replacing the multi-bit reuse counter with a single seen-in-stash bit is a metadata-cost experiment, and the hit-rate result suggests the counter's extra states carried some signal beyond the second-hit gate. The claim as written is not confirmed: metadata-size and per-op-time were not recorded in the fitness vector, so the cost half of the hypothesis is unmeasured. Caveat: the overlay sources could not be read in this pass, so the mechanism above is inferred from the hypothesis text rather than verified against the code diff.
