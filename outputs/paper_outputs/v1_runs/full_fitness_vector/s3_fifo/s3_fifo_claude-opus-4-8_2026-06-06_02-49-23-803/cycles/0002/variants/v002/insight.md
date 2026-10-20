# Variant v002 Insight

## Helped

- (none) per-op-time `get`/`put` shifts are within prior measurement noise and not attributable to ghost sizing; the resident scan path is unchanged.

## Hurt

- hit-rate: doubling the ghost queue to `main_size * 2` regressed hit-rate rather than raising it — the wider reuse-distance window readmitted keys whose reaccess came too far apart, promoting stale long-distance keys into main and displacing more recently reused entries, contradicting H02's premise.
- metadata-size: the new `ghost_size` field enlarged the `S3Fifo` cache struct (112→120 bytes); the dimension summary stays at the `Entry` size (24) since ghost still stores keys only, but the per-instance control struct grew for no benefit.

## Takeaway

On workload-a a larger ghost is not a free win: stacking a doubled ghost on v003's `capacity/10` split lowered hit-rate, so the split and ghost knobs are not additive — v003's gain came from main-queue capacity, and widening reuse detection here mostly proves longer-distance reuse, which this trace does not reward.
