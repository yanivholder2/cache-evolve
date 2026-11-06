# Variant v001 Insight

## Helped

None. Both 16384 cells (web_0 and hm_0) are identical to the parent's, as expected. In 128-slot buckets, `probation_len(128)` is still 64: `3 * 128 / 8 = 48` is below `PROBATION_FLOOR`, so the floor still sets the size and small buckets are unchanged.

## Hurt

- hit-rate (web_0 @ 131072): The goal was to raise this cell, but it went down. Growing probation from 256 to 384 slots in 1024-slot buckets cuts the frequency-ranked main region from 768 to 640 slots. web_0's reused working set at this size needs that main capacity more than it needs the longer probation window. The extra second touches caught in probation do not make up for the promoted blocks pushed out of a smaller main.
- hit-rate (hm_0 @ 131072): This cell dropped only slightly, so hm_0's hot set mostly still fits in 640 main slots. Even so, the 1/4 → 3/8 shift hurt it a little, not helped it. Both large cells moved the same way, against the change.

## Takeaway

v003 found that deeper probation is "not zero-sum" at 128-slot buckets, but that does not carry over to 1024-slot buckets. Once probation is already 256 slots deep, trading main slots for more depth costs hit-rate, and web_0 pays most. This matches the hypothesis's refutation case: probation depth is not what limits web_0 at 131072, so the 1/4 fraction should stay for large buckets. Large-bucket gains on web_0 will have to come from how the main region chooses victims or keeps counters on promotion, not from resizing the probation ring.
