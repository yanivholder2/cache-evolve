# Variant v001 Insight

## Helped

- **per-op-time (put p99)**: Tail latency improved 3% (177→172 ns) from inline bucket node storage reducing pointer chains in the rare high-saturation case, though this benefit is negligible on an underutilized workload.

## Hurt

- **hit-rate**: Cache efficiency decreased 2.3% (0.6904→0.6741) because the more complex bucket header logic introduced subtle changes in eviction candidate selection order when switching between simple and bucketed modes.
- **per-op-time (get p50)**: Median get latency regressed 35% (34→46 ns) due to added conditional logic in the bucket header's inline/overflow management during the common-case simple-mode operations.
- **per-op-time (put p50)**: Median put latency regressed 10% (72→79 ns) from the same cause—every put calls complex unlinking/linking logic that branches on inline vs overflow storage, even when operating in simple mode.

## Takeaway

The cache-locality bucket optimization targeted the wrong hot path. While packing the first 2-3 node indices inline reduces pointer-chasing during bucketed-mode eviction scans, this workload operates almost entirely in simple mode (as v003's history confirms), making the optimization invisible. Meanwhile, the added complexity in bucket header management—checking inline slots, promoting overflow nodes, updating multiple pointers—taxes every get and put operation, including those in the common simple-mode path where no bucketing infrastructure is used. The fix assumes saturation is frequent; when it never arrives, you pay only the cost. This variant demonstrates that optimizing latency-critical hot-path code requires profiling the actual execution path, not the code path, especially in adaptive systems where multiple code branches compete for the same hot cycle budget.
