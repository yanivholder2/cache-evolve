# Variant v003 Insight

## Helped

## Hurt

- hit-rate: The overlay source still keeps `freq: u8` inside `Entry` and has no `small_freq` or `main_freq` deques, so the layout change H03 describes did not land; the eviction logic is the v002 logic, and the drop relative to the parent cannot be attributed to the hypothesis's mechanism.

## Takeaway

H03 was meant to move `freq` out of `Entry` into parallel deques while keeping admission and eviction identical, but the v003 overlay does not contain that edit, so this variant tests nothing about metadata size or the parallel-deque design. The hit-rate drop relative to v002 is therefore unexplained by this change and should be investigated as a possible overlay or build mismatch before any conclusion is drawn about the layout idea. Metadata-size and per-op-time were not reported, so the hypothesis's central question remains open.
