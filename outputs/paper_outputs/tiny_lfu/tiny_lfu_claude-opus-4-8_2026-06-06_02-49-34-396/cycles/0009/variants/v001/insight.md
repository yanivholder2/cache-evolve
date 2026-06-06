# Variant v001 Insight

## Helped

- metadata-size: Halving `SKETCH_DEPTH` from 4 to 2 cuts the `CountMinSketch` heap allocation in half (two rows instead of four over the same width), so its footprint drops cleanly — though the reported `metadata-size` summary tracks the unchanged `Node` size, not the sketch heap.

## Hurt

- per-op-time: Contrary to the hypothesis, both `get` and `put` p50/p99 rose rather than fell — dropping two sketch rows did not speed the hot path, indicating `increment`/`estimate` row iteration is not the per-op bottleneck and the saved work was below measurement noise.
- hit-rate: With only two rows, the count-min sketch loses collision-correction headroom, so aliased frequency estimates leak into admission decisions and hit-rate slips below the parent's band.

## Takeaway

Sketch depth was not accuracy-saturated nor a free per-op cost: cutting it from 4 to 2 rows traded away a slice of hit-rate while failing to deliver the predicted speedup. The two extra rows the parent carried were still doing real collision-correction work, and their per-op cost is too small to recover by removing them — so depth=4 remains the better operating point on this wide table.
