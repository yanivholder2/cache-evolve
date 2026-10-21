# Variant v003 Insight

## Helped

- (none) No dimension improved. I could not inspect the overlay source in this session, so this rests on the hypothesis and the fitness vectors only.

## Hurt

- hit-rate: Remembering a per-key frequency flag in the ghost and re-admitting flagged keys to main with freq=2 did not beat the baseline. It came out marginally lower, which is within noise of "no change".
- metadata-size: The variant's fitness vector has no metadata-size dimension, so the cost of changing the ghost from a key set to a key -> flag map was not measured. The extra per-ghost-entry flag is a plausible cost.
- per-op-time: This dimension is also missing from the variant's vector, so the cost of the richer ghost bookkeeping on `put` is unknown.

## Takeaway

On workload-a at this cache size, the baseline's plain ghost-hit-to-main admission already captures nearly all the recoverable reuse. Recording whether a key was hit while in small, and boosting it to freq=2 on a ghost hit, added state without adding signal. Future variants should not spend more ghost metadata on per-key frequency. This variant also lacks the metadata-size and per-op-time measurements, so it can't be compared with the baseline on those dimensions.
