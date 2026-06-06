# Variant v003 Insight

## Helped

- per-op-time: capping the second-chance loop at `SECOND_CHANCE_CAP = 8` shaved a hair off `put` p50, since the bounded decrement-and-rotate budget shortens the median `evict_main` traversal slightly.

## Hurt

- hit-rate: forcing eviction once `chances` hits the cap drops entries that still have `freq > 0`, evicting a few reuse-proven keys the parent's unbounded loop would have rotated and kept — a small but real loss.
- per-op-time: the intended win, `put` p99, did not move (the tail stayed flat), because on workload-a's freq distribution the cap almost never fires, so the worst-case O(main) rotation it was meant to bound rarely occurs in this trace anyway.

## Takeaway

Bounding `evict_main` rotations to a constant is a cure for a pathology this trace doesn't exhibit: the put tail was never driven by long second-chance scans, so the cap buys no p99 improvement while its forced evictions of `freq > 0` entries cost a sliver of hit-rate. The mechanism only pays off on workloads where the main queue is densely populated with high-freq residents — not workload-a at capacity 2048.
