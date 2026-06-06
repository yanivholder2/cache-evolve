# Cross-Cycle Insights

## Recurring wins

- **Bucket indexing for O(1) eviction**: Cycles 0002, 0007, 0009 confirm that structural indexing (bucket pointers, sorted frequency lists) enables constant-time candidate selection. Removal of this infrastructure (Cycle 0009 V002) caused 215% put p99 regression from O(N) eviction scanning, despite maintaining hit rate. Bucket indexing is load-bearing for latency, not purely for selectivity.

- **Algorithmic queue-flush optimization**: Cycle 0007's batched queue-flush with two-batch swap strategy achieved simultaneous improvement of both get p99 (42%) and put p99 (50%)—the first design to break the operation-shifting trade-off. Success came from algorithmic optimization of the flush mechanism, not conditional deferral logic.

- **Bounded-window search spaces**: Cycles 0002–0003 show that fixed-size scan windows (e.g., min(capacity/4, 256) nodes) eliminate pathological O(N) tail latency while maintaining hit rate. Structural partitioning of the eviction search space proved more latency-predictable than selective scanning across unbounded lists.

- **Hit rate is determined by window size, not eviction sophistication**: Cycles 0002–0010 show consistent hit-rate plateau (~0.66–0.69) regardless of whether eviction uses frequency, recency, time-windowing, or pure LRU. Window-size calibration alone captures selectivity gains; frequency tracking provides O(1) indexing, not incremental selectivity.

- **Conditional activation beats uniform overhead**: Cycle 0003 V003 (saturation-aware mode switching) achieved 34% get p99 improvement by remaining in lightweight mode for most operations, activating expensive logic only under high saturation. Uniform application of expensive operations (Cycles 0008, 0010) consistently regressed median latencies.

## Recurring failures

- **Intrusive hot-path logic for deferred-work patterns**: Cycles 0006–0008 show that checking drain thresholds or saturation conditions on every get/put adds measurable overhead that often outweighs the benefit of deferring work to rare code paths. Cycle 0006 V002 achieved 59% get p99 improvement by completely removing drain checks; incremental draining (Cycle 0005 V002) added 32% regression despite amortizing work.

- **Dynamic allocation for mode-switching**: Cycle 0005 V001's lazy bucket initialization via `Option<Box<>>` created catastrophic overhead (40× get p99 spike) from repeated allocation/deallocation at saturation boundaries. Pre-allocation, while heavier on underutilized caches, avoids the pointer indirection and allocation cost that mode-transition oscillation incurs.

- **Asynchronous or sampled feedback in adaptive systems**: Cycles 0009 V001 (sampled saturation tracking) and multiple earlier attempts show that delaying feedback for arithmetic optimization backfires. Operating in incorrect modes for N operations has multiplicative cost; real-time feedback, even if slightly costly, is preferable. Stale saturation state (Cycle 0009) degraded both get and put p50 despite saving computation.

- **Unused infrastructure on light-load workloads**: Cycles 0008 V001, 0010: batching and deferral machinery optimizes rare expensive paths but adds bookkeeping cost to every operation. On workloads that never reach high saturation or sustained load, these optimizations become pure overhead. The tested workload operates below 70% utilization; infrastructure assuming high saturation or frequent promotions is misaligned.

- **Frequency tracking removal breaks latency predictability**: Cycles 0008 V002, 0009 V002 demonstrate that removing frequency tracking in pursuit of simpler eviction causes severe latency regression (215% put p99 in Cycle 0009, 203% get p99 in Cycle 0008) due to loss of O(1) eviction infrastructure. Frequency buckets are not redundant with window size; they provide critical structural support for constant-time candidate discovery.

- **Single-axis optimization hits local equilibrium**: Cycle 0010 shows that complementary improvements (cache-locality packing, quantized frequency levels, opportunistic buffering) all trade median latency for tail improvements at unfavorable ratios on this workload. Cycle 0007's baseline is tightly tuned; improving one metric requires accepting regression elsewhere.

## Open questions

- **Hit-rate ceiling and workload saturation**: Hit rate has plateaued at ~0.66–0.69 across all eviction strategies since Cycle 0002. Is window size already optimal, or does this workload's Zipfian skew limit further selectivity? Empirical saturation profiling would reveal whether optimization landscape shifts at higher load—all saturation-aware variants assume dynamics that may not materialize on this trace.

- **Frequency tracking trade-off: selectivity vs. infrastructure**: Cycles 0009–0010 suggest frequency buckets provide O(1) eviction structure, not selectivity. Does increasing window size further validate that window calibration alone captures hit-rate gains? Can hit-rate parity with pure FIFO be confirmed, eliminating debate over frequency necessity?

- **Median latency recovery without sacrificing tail improvements**: Cycle 0007's batched queue-flush achieved breakthrough tail-latency gains but introduced 20% get p50 regression. Can batch size, drain frequency, or threshold tuning recover median latency, or is the regression an inherent trade-off?

- **Generalization beyond read-heavy, underutilized workloads**: All cycles tested a single light-load, read-dominated trace. Do Cycle 0007's baseline and its variants maintain the same trade-offs under write-heavy synthetic workloads, sustained high saturation, or longer episodes with changing access patterns? Unknown whether the local optimum shifts with workload class.

- **Orthogonal optimization axes**: Eviction strategy and saturation-tracking axes appear exhausted after 10 cycles. Are memory hierarchy (cache-line alignment, prefetch-friendly traversal), lock-free contention reduction, or parallelism via sharding the remaining frontier for improvement?

## Per-policy notes

- **Frequency-recency hybrid (Cycles 0001–0002)**: Initial selective-eviction approach exposed the O(N) eviction-scan bottleneck. Cycle 0002's bucket indexing (V002) solved the structural problem and established the core stable design pattern: bounded-window search paired with frequency buckets for O(1) indexing.

- **Bounded-window + frequency bucket baseline (Cycles 0002–0010)**: The most durable design. Evolved from simple bucket indexing → saturation-aware mode switching (Cycle 0003) → batched queue-flush optimization (Cycle 0007). No fundamental replacement found. Bucket indexing proved load-bearing after Cycle 0009 attempted removal.

- **Spatial zone partitioning (Cycle 0001)**: Provided latency-friendly structure via pre-partitioning but offered no hit-rate benefit. Subsumed by bounded-window eviction, which achieved both latency predictability and hit-rate parity with selectivity.

- **FIFO and pure LRU variants (Cycles 0002, 0007–0009)**: Multiple attempts to eliminate frequency tracking via pure FIFO or recency-based selection maintained hit rate (~0.6652 parity) but failed to improve latency or incurred catastrophic cost (Cycle 0009: 215% put p99 regression). Validates that window size is the selectivity parameter; frequency tracking's value is O(1) infrastructure, not algorithmic selectivity.

- **Deferred and batched promotion mechanics (Cycles 0004–0010)**: Core optimization axis. Evolved from per-operation deferral → incremental amortization → Cycle 0007's batched queue-flush (algorithmic optimization). Further attempts to reduce overhead (partial amortization, async sampling, deferral at saturation boundaries) all regressed net metrics. Cycle 0007's design represents the sweet spot: deterministic algorithmic batching without intrusive conditional checks.

- **Saturation-aware mode switching (Cycles 0003–0010)**: Architecturally sound. Cycle 0003 V003 succeeded by activating expensive logic only under high saturation. Subsequent attempts to reduce overhead (async sampling in Cycle 0009, lower thresholds in Cycle 0008, conditional deferral in Cycle 0005) all failed; the saturation check itself is cheap, but stale feedback or unused infrastructure are expensive. Correct design requires workload validation that assumed saturation levels actually occur.
