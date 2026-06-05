# Variant v003 Insight

## Helped

- hit-rate: Periodic frequency decay (right-shift at `capacity * 16` operations) allows the cache to gradually deprioritize ancient hotspots, enabling newly-popular items to gain relative weight without losing historical discrimination.

## Hurt

## Takeaway

Periodic frequency decay with a threshold of `capacity * 16` operations offers modest but consistent improvement over pure frequency-based eviction on shifting workloads. The algorithm halves all frequency counters at regular intervals, preserving relative item ordering while fading historical bias. The small gain suggests this workload has detectable phase shifts where early hotspots become less relevant, but the shifts are gradual enough that the original frequency-based approach already captures most value. The cost-to-benefit ratio remains favorable: decaying frequencies improves hit rate at the cost of O(capacity) work only every `capacity * 16` operations.
