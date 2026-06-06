# Variant v002 Insight

## Helped

- **hit-rate**: Frequency tracking removed and window size increased to min(capacity/2, 512) confirms the hypothesis that Zipfian skew is captured by window size alone; hit rate is stable without frequency-awareness overhead.

- **per-op-time (put p50)**: Simpler FIFO eviction logic without frequency field lookups or counter increments reduces average put latency from 75ns to 72ns.

## Hurt

- **per-op-time (get p50)**: Removal of frequency metadata or optimization passes results in slightly higher average get latency (41ns → 43ns), though tail latency improves.

- **per-op-time (put p99)**: Increased variance in put tail latency (135ns → 141ns) suggests the bounded-window FIFO eviction order introduces less predictable contention or branching patterns than the prior strategy.

## Takeaway

Variant v002 validates a critical insight: on this Zipfian workload, hit rate plateaus at a window size independent of eviction strategy sophistication. Removing per-item frequency tracking and switching to FIFO with a 2× larger window maintains hit rate while simplifying the cache, but trades improvement in average put latency for increased tail latency variance—indicating that simplification did not uniformly reduce contention across all operation paths.
