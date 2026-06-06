# Cycle 0006 Insight

## Cohort outcome

Three strategies for optimizing the Cycle 0005 v002 baseline were tested: combining saturation-aware activation with incremental queue draining (V001), removing drain-trigger logic entirely from get() to prioritize the hot path (V002), and replacing frequency counters with operation-count-based recency to eliminate per-operation tracking overhead (V003). V001 and V002 both won: V001 improved put p50 by 18% (72→59 ns) via amortization, while V002 dramatically improved get p99 by 59% (151→62 ns) by eliminating drain checks from the fast path. V003 failed by introducing a new latency cliff during eviction-candidate search despite removing counter overhead.

## Patterns

- **Incremental draining on the hot path incurs overhead that exceeds its benefit**: V001's attempt to amortize deferred work by checking and draining the queue during get() added checking logic, validation, and potential batch-draining on the critical path, causing get p99 to regress 34% (97→130 ns). The median put benefit (18%) came at the cost of tail latency degradation where it matters most. This teaches that amortization strategies must be evaluated for their critical-path impact, not just average-case amortization; a single drain-check on every get() is more expensive than accepting rare bulk flushes at put().

- **Complete deferral outperforms partial amortization for tail latency on the hot path**: V002 proved that V001's regression was caused by drain-trigger checks intrusive to get(). By eliminating all drain responsibility from get() and flushing the entire queue at put(), V002 achieved a pristine get fast path (get p99 improved 59% to 62 ns) while keeping put p99 competitive (191 ns vs. parent's 173 ns). The trade-off is cleaner: prioritize the operation executed far more frequently (get) by accepting occasional bulk work on the less-frequent put().

- **Frequency/recency-tracking search within bounded windows introduces new latency cliffs**: V003 attempted to eliminate per-operation frequency counter overhead by replacing it with operation-count-based recency (`last_access_count` field). However, the eviction candidate search—scanning the bounded window to find the minimum operation count—introduced cache misses and pointer chasing that create a severe put p99 regression. The bounded-window sequential search from earlier cycles (which implicitly selects LRU items) is cheaper than an explicit min-search, even when that search operates on a small fixed window.

- **Metadata overhead persists even when the underlying operations are simplified**: V003 added only 16 bytes total (8 per node, 8 per cache struct) compared to V001's 48 bytes, yet the search operation that justifies this metadata was more expensive than the counters it replaced. V001's larger metadata overhead at least enables an operation (incremental draining) that has some benefit on the median; V003's leaner metadata enabled a worse operation (min-search on window).

- **Hit rate remains stable across all variants, confirming selectivity plateau**: V001 did not move hit rate (inherited from parent), and V003 maintained 0.6652 hit rate despite the fundamental shift from frequency to recency. This reinforces the pattern from Cycle 0005: the bounded-window baseline is already selecting near-optimal candidates for this workload; further eviction-selection sophistication does not materially improve selectivity.

- **Hot-path minimalism beats architectural complexity**: V002's straightforward design—queue promotions on get, flush on put, no intermediate checks—beats V001's sophisticated saturation-aware amortization and V003's operation-count optimization. The cohort demonstrates that reducing intrusive logic from the most-executed operation (get) is more impactful than clever work distribution or alternative tracking mechanisms.

## Open questions

- Is V002's put p99 regression (173→191 ns) and put p50 degradation acceptable in production workloads with higher write ratios? The current test workload is read-heavy; a write-heavy scenario may reverse the trade-off value.

- Can V001's drain-trigger threshold or batch-size be aggressively tuned to reduce get-path overhead without sacrificing the put p50 improvement? The current design uses checking on every get; could sampling or deferred threshold checking reduce overhead?

- Does V003's latency cliff during eviction candidate search stem from the search algorithm itself (cache misses on window scan) or the operation-count tracking overhead? Could a different recency signal (e.g., approximate/sampled last-access) avoid the search cost?

- What is the hit-rate ceiling for this workload with the bounded-window baseline? Multiple cycles have confirmed that frequency, recency, and count-based tracking do not improve hit rate; is the workload's Zipfian skew or the window size already optimal?

## Continuation

V002 emerges as the winning design for this cycle and should become the new baseline: it achieves the strongest improvement on the most critical metric (get p99) by maintaining a clean hot path, while accepting moderate put latency cost. V001 demonstrated that amortization techniques on the critical path are self-defeating; the checking and batching overhead exceeds the benefit when operations are underutilized. V003 showed that eviction-selection search, regardless of what signal it optimizes, is a latency bottleneck that alternatives to frequency counting do not solve.

Next cycle should: (a) integrate V002 as the baseline and explore put-side optimizations—can the deferred promotion queue be flushed more efficiently, or can multiple consecutive puts be coalesced to amortize queue processing? (b) stress-test V002 on longer traces and write-heavy synthetic workloads to validate that the get p99 gain holds and put p99 regression is acceptable for the target use case; (c) investigate fundamentally different eviction strategies (e.g., simple LRU with a larger fixed window, TTL-based eviction, or statistical sampling) that avoid the overhead of per-operation tracking or windowed searches entirely; or (d) measure whether the workload has sufficient frequency skew to benefit from any selective eviction at all—if simple FIFO within a large window matches selective-eviction hit rate, the cost of selectivity is unjustified.
