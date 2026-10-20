# Variant v003 Insight

## Helped

## Hurt

## Takeaway

H03 swapped the arena links and head/tail indices from `Option<usize>` to a `u32` with a `u32::MAX` sentinel, aiming to shrink each node from 48 to 24 bytes without changing eviction order. The hit-rate dimension did not move from the parent, which is what the list-algorithm-is-unchanged claim predicts. However, v003's fitness vector reports only hit-rate, so the metadata-size and per-op-time effects of the node shrink are unmeasured and this variant cannot yet be credited with the size win it targets. The source overlay could not be inspected in this run, so the change itself is taken from the hypothesis rather than verified against the code.
