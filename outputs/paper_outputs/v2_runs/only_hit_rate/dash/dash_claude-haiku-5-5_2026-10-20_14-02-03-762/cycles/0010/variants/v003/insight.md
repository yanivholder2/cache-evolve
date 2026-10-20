# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The hypothesis gates stash promotion on a second hit that falls within a window W of the item's first-seen index, and resets first-seen for late second hits instead of promoting them. Any item re-referenced after the window is kept out of the MRU landing, which removes promotions the parent would have made on workload-a at size 2048. (Reasoning is from the hypothesis text; I could not read the overlay source to confirm the window check is implemented this way.)

## Takeaway

On workload-a at size 2048, requiring the second hit to be temporally close to the first made hit-rate worse than the parent, so the claim that second-hit recency carries promotion signal is not supported on this trace and size. This is one trace at one size. I could not read the overlay sources or the manifest in this pass (the search and shell tools were unavailable and the overlay path is a directory I could not list), so I have not confirmed that the recency check, the clock, or the W constant match the hypothesis as written. Treat this as a reading of the fitness result against the stated change, not of the implementation.
