# Variant v002 Insight

## Helped

- **per-op-time (get p99)**: Removing the drain-trigger check and batch-processing logic from get() eliminates intrusive overhead in the hot path, reducing tail latency by 59% (151 ns → 62 ns) and restoring get performance nearly to baseline.
- **per-op-time (get p50)**: The cleaner, interrupt-free get() fast path maintains median latency at 35 ns without any amortization overhead.

## Hurt

- **per-op-time (put p99)**: Deferring all promotion flushing to put() means the entire accumulated queue is processed at once before eviction, increasing put tail latency by 10% (173 ns → 191 ns).

## Takeaway

By eliminating drain-during-get entirely, this variant proves the hypothesis: the drain-check and batch-draining logic was the source of the 151 ns get p99 regression in the parent. A completely deferred approach—queuing in get() with flush-all in put()—produces a dramatically cleaner get fast path (62 ns p99), though at the cost of accepting higher put p99 spikes (191 ns). The trade-off favors systems where get latency is critical and put tail latency can absorb occasional bulk flushing, demonstrating that simple deferral (no amortization) outperforms partial amortization for tail latency on the hot path.
