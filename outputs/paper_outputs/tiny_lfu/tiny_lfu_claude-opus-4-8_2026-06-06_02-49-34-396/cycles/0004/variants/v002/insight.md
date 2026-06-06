# Variant v002 Insight

## Helped

- hit-rate: Quadrupling `SKETCH_WIDTH_FACTOR` (4 -> 16) widens the count-min table from width-8192 to width-32768, roughly quartering hash collisions so cold-key counts stop being inflated — sharpening the `incoming_freq > victim_freq` admission test and the sampled-coldest victim choice enough to clear the parent's hit-rate.

## Hurt

- (none) The larger table did not measurably slow operations: `get`/`put` p50 stayed flat and p99 even ticked slightly lower, so the feared per-op cost of a bigger sketch never materialized at this width.

## Takeaway

For this workload the count-min sketch was collision-starved at factor 4, not policy-limited: simply giving each row 4x more columns recovers a real hit-rate gain with no per-op penalty, since the extra width only enlarges a heap table the hot path already strides through with one cached double-hash. The reported metadata-size axis is unaffected because it measures per-entry `Node` bytes, not the sketch heap — so this gain is essentially free against the tracked budget.
