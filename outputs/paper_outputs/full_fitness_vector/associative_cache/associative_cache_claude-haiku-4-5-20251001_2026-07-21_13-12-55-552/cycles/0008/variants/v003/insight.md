# Variant v003 Insight

## Helped

- hit-rate: Predictive insertion bias based on recent hit/miss history successfully positions new items closer to retention when the workload enters a high-reuse phase, improving hit probability by steering hot items to the head of the bucket's eviction order.

## Hurt

- per-op-time: Recording hit/miss outcomes in the buffer on every get and checking the buffer on every put introduced latency overhead; median get time increased by 45% and median put time by 40%, offsetting the modest hit-rate gain.
- metadata-size: The 16-element rolling buffer and circular index add 40 bytes to the cache struct, increasing base memory footprint without per-item expansion.

## Takeaway

Learned insertion positioning improves hit-rate by exploiting temporal locality in access patterns without expanding per-item metadata, but the cost of maintaining and polling the buffer on every operation outweighs the modest hit-rate gain (0.43%) at the measured latency sensitivity. The approach proves that algorithmic adaptivity can improve cache behavior, but the overhead must be amortized over many more accesses to justify the 40–50% median latency increase.
