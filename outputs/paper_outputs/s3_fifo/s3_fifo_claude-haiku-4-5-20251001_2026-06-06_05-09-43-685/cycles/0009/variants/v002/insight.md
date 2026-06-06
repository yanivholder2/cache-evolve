# Variant v002 Insight

## Helped

- **per-op-time (get)**: Moving occupancy computation from the hot put path to the cold ghost-hit re-admission path eliminates per-operation latency overhead, improving get p50 by 25ns (19% faster) without branching on every operation.
- **per-op-time (put)**: Keeping occupancy sensing off the hot put path preserves put performance, with a marginal 3ns improvement as fewer eviction cascades occur on the main queue.

## Hurt

- **hit-rate**: The occupancy-responsive boost formula `(2.0 + occupancy_ratio²)` gives ghost-hit re-admissions excessive protection (freq 2–3), causing them to accumulate in the main queue and displace items from the small-queue working set before they can graduate, reducing hit-rate by 1.55%.

## Takeaway

This variant validates that occupancy-aware adaptation can be effectively relocated to cold paths to eliminate hot-path overhead—achieving a significant 19% get latency improvement. However, the boost formula is miscalibrated: re-admitted items with freq 2–3 survive too many eviction cycles, starving the small-queue promotion pipeline that drives the baseline algorithm's hit-rate. The core architectural insight holds (cold-path computation is free), but the signal—boosting ghost hits based on occupancy—requires either lower boost magnitudes or a more selective re-admission criteria to avoid crowding out the working set.
