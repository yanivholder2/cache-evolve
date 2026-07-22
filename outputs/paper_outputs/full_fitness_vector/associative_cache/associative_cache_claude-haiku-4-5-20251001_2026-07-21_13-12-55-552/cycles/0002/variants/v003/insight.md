# Variant v003 Insight

## Helped

- hit-rate: FIFO maintained the 0.6611 hit-rate despite pure insertion-order eviction, showing that recency-based replacement is not strictly necessary for Zipfian workload patterns on this cache geometry.

## Hurt

- per-op-time (get): Switching to FIFO increased get latency from 69ns to 100ns, contradicting the hypothesis that eliminating bookkeeping would reduce per-hit overhead.
- per-op-time (put): Put latency increased from 143ns to 200ns, indicating FIFO's implementation costs outweigh any savings from skipping recency tracking.

## Takeaway

FIFO performed a counterintuitive trade-off: it preserved hit-rate but degraded latency by ~40-45%, the opposite of what the hypothesis predicted. This suggests that the theoretical cost savings of "zero bookkeeping on hit" do not materialize in practice at this cache geometry. Either FIFO's implementation carries hidden overhead, or the CPU-level optimizations that make ClassicLRU fast (e.g., branch prediction, cache coherency patterns from move-to-front) outweigh the cycles saved by skipping position updates. The finding narrows the frontier: minimalism alone does not guarantee speed—implementation quality and workload-specific micro-architectural fit matter more than theoretical simplicity.
