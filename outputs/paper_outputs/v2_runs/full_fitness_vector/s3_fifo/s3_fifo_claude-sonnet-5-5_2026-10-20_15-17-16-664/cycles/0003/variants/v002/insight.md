# Variant v002 Insight

## Helped

- hit-rate: Cutting the small queue from capacity/10 to capacity/20 moves more slots to main, and the ghost queue (sized to main_size) grows with it. That keeps more re-referenced objects resident. The gain is tiny, so the 5% setting is only marginally better than 10% on workload-a.

## Hurt

- (none) metadata-size is unchanged, because only the split of a fixed capacity moved. per-op-time also moved, but the edit is a single constant and adds no work to get or put. The movement is most likely run-to-run noise, so it should not be credited to or blamed on the change.

## Takeaway

The small-queue ratio shows diminishing returns. Going from 25% to 10% gave a small gain, and going from 10% to 5% gave an even smaller one, which suggests the curve is flattening near the low end on this trace. Further shrinking is unlikely to pay off unless it is tested on other workloads. Gains from this knob are small enough that they need repeated runs and more traces to separate from noise.
