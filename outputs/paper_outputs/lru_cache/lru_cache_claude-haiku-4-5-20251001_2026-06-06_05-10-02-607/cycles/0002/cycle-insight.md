# Cycle 0002 Insight

## Cohort outcome

Two distinct structural approaches to fixing the eviction scan bottleneck from Cycle 0001 were tested: bounded search windows (V001) and frequency-level bucket indexing (V002). Both Winners successfully eliminated pathological O(N) eviction latency spikes, with V001 achieving 3.2× put p99 improvement via simplicity, and V002 achieving 15.6× improvement while also gaining hit rate (0.6904 vs 0.6721). The hybrid zone+frequency approach (V003) failed due to expensive scanning overhead that contradicted its stated latency-robustness goal.

## Patterns

- **Eviction data structure choice dominates latency predictability**: V001's bounded window (scan first min(capacity/4, 256) nodes) and V002's bucket indexing both achieve constant-time eviction discovery, but through opposite design philosophies—V001 constrains the search space structurally, V002 pre-organizes metadata. Both succeed where V003's linear scan fails, showing that the implementation approach matters more than the eviction strategy itself.

- **Hit rate and tail latency are now decoupled**: V002 improved both hit rate and put p99 simultaneously, while V001 held hit rate constant while fixing latency. This suggests that Cycle 0001's perceived trade-off between selective eviction and latency was actually due to inefficient scanning, not fundamental conflict between the goals.

- **Normal-case latency increases with bucketing complexity**: V002 paid a measurable cost in get p99 (67→147 ns) and put p50 (63→70 ns) due to bucket management overhead, while V001 maintained virtually flat median latencies. This indicates that frequency-aware eviction via buckets adds pointer-chasing work even on the fast path.

- **Zone-based partitioning without efficient data structures is too expensive**: V003 attempted to combine V001's zone structure with frequency tracking in the main zone but suffered severe put latency degradation (57–812% slower) due to bounded scanning (20 nodes). The bounded window itself was insufficient; only when paired with bucket indexing (V002) did the strategy pay off.

## Open questions

- Does V001's bounded window maintain hit rate gains over longer runs or on non-Zipfian workloads where the first min(capacity/4, 256) nodes are not representative of hot items?
- Can V002's frequency bucket overhead be reduced (e.g., via adaptive promotion thresholds or lazy bucket updates) to close the get p99 regression without sacrificing hit rate?
- Which Winner generalizes better under cache saturation, sudden access pattern shifts, or mixed read-heavy vs. write-heavy phases?

## Continuation

Both Winners solved the critical eviction bottleneck identified in Cycle 0001, but through different trade-offs: V001 prioritizes latency predictability and simplicity at the cost of selective eviction precision, while V002 gains hit rate and tail latency at the cost of normal-case overhead. The data structure choice (bounded search vs. buckets) emerges as more critical than the eviction strategy itself. Next cycle should determine which trade-off better serves production: (a) stress-test V001's bounded window on longer or more irregular workloads to confirm hit rate stability, (b) optimize V002's bucket organization (e.g., frequency decay tuning, lazy updates, or O(1) bucket lookup via adaptive sizing) to reduce get p99 overhead, or (c) explore hybrid layering where V001's simplicity serves as the baseline and V002's bucketing is applied selectively only when hit rate is critical. The path forward depends on whether V001's hit rate holds under adversarial access patterns.
