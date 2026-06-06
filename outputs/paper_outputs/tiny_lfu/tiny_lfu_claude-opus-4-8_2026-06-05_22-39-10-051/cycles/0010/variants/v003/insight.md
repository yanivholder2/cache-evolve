# Variant v003 Insight

## Helped

- (none): The only measured dimension is hit-rate, and halving `SKETCH_WIDTH_FACTOR` from 64 to 32 did not raise it — the change's intended payoff (≈2x smaller sketch `Vec<u32>` rows) is a memory win this fitness vector does not score.

## Hurt

- hit-rate: A one-ten-thousandth dip versus the parent — indistinguishable from noise, meaning a 32x-cells-per-capacity sketch still absorbs essentially all count-min collisions on this low-churn, popularity-stable trace.

## Takeaway

Cutting the sketch width factor to 32 holds hit-rate flat while halving estimator footprint, confirming the 64x sketch was over-provisioned by at least 2x on workload-a. This pins the free-trim boundary at or below 32x — between the refuted 4x cut (cycle 0007) and the previous 64x winner — so width can be reclaimed for memory without touching accuracy here.
