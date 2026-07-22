# Cross-Cycle Insights

## Recurring wins

- **Simple policy beats complex policy**: ClassicLRU's straightforward recency-based eviction consistently outperforms or matches all attempts at sophistication (LFU, TimestampLRU, FIFO with victim selection, segment-based tracking). Hit-rate differences are negligible across all policy variants tested, indicating the Zipfian workload is insensitive to eviction strategy details beyond basic recency.

- **Micro-optimization of the hit path succeeds where policy change fails**: Cycle 0004's v003 (early-exit on key match, 64-byte cache-line alignment on ClassicLRU) is the first variant to achieve a net Pareto win—hit-rate maintained at 0.664, latency improved 33–50%. All prior cycles show policy changes regressed latency by 40–400%, while v003 proves implementation-level cleverness (not algorithmic change) is the path forward.

- **Cache locality matters more than asymptotic complexity**: Consistent pattern across cycles: O(n) vector operations on 16-item buckets outperform O(1) timestamp updates, O(1) position lookup fails when replaced with O(n) bucket scan for victim selection. The classical big-O analysis inverts under real hardware—ClassicLRU's small-vector ops have superior cache coherency and compiler optimization relative to pointer-chasing or metadata maintenance.

- **Geometry trade-off can be salvaged through implementation detail**: Cycle 0003 revealed that wider buckets (32×64) suffer catastrophic tail latencies due to linear intra-bucket search cost. Cycle 0004 showed that early-exit and cache-line alignment eliminate that tail-latency crisis, making wide geometry viable for hit-rate gain without sacrificing latency.

## Recurring failures

- **Optimizing the eviction path at cost of hit-path performance is a losing trade-off**: Variants in cycles 0001–0004 that reduced eviction complexity (O(1) removal, segment tracking, victim-selection optimization) all regressed latency by 40–4000%, because on Zipfian workloads, hits vastly outnumber evictions. Spending per-operation cost to save infrequent eviction scans inverts the priority. v003's success comes from flipping this: optimize the hot hit path, not the rare eviction path.

- **Narrow bucket geometry backfires despite reducing scan depth**: Theory predicted that shrinking buckets from 16 to 8 items would reduce linear-search cost and improve latency. Cycle 0003's v001 (256×8) regressed both hit-rate and latency by ~45%, indicating that hash-function distribution overhead and memory-layout fragmentation across 256 buckets outweigh any benefit from shallower scans. This repeats cycle 0001's pattern: asymptotic advantages vanish under practical constraints.

- **Unvetted policy pivots fail to integrate or validate**: Cycle 0003's H02 hypothesis (FIFO with 64×32 geometry to decouple latency from bucket width) was sound but the variant failed to execute the policy change. This left a critical testable claim unresolved and signals that without careful build/integration checks, policy hypotheses cannot be validated. Cycle 0004 revisited FIFO with explicit victim-selection optimization, but the change regressed latency 2–4×, suggesting the integration issue was not the root problem but rather that FIFO's per-hit overhead under wide buckets is inherently expensive.

- **Theoretical O(1) or O(log n) complexity improvements collapse in practice**: Every cycle contains a variant claiming asymptotic advantage over ClassicLRU (Swap's O(1) removal, TimestampLRU's O(1) updates, segment-based LRU's bounded eviction). All regressed latency, often severely. The constant factors and cache-level micro-architecture dominate the benchmark scale and workload, rendering complexity analysis a poor predictor of actual performance.

## Open questions

- **Is v003 the true Pareto frontier or a local optimum?** Cycle 0004 v003 achieved hit-rate 0.664 with controlled latency, an improvement over cycle 0003's wider-bucket variants. But how does its overall fitness rank relative to all prior cycles? Is there a further-out Pareto point?

- **Do v003's micro-optimizations (early-exit, cache-line alignment) generalize beyond 32×64 geometry?** Could these techniques improve performance on narrower buckets (e.g., 128×16 or 256×8), effectively unlocking the asymptotic benefit of reduced scan depth while keeping hit-path cost low? Or are they specific to wide-bucket scanning efficiency?

- **What is the true bottleneck of FIFO when applied to wide buckets?** Cycle 0002's zero-bookkeeping FIFO achieved the same hit-rate as ClassicLRU (0.6611) on baseline geometry. Cycle 0004's FIFO with timestamp-based victim selection regressed 2–4× latency. Is the cost in victim selection (O(n) scan) or in the policy semantics under wide buckets? A correctly integrated, zero-bookkeeping FIFO on 64×32 geometry remains untested.

- **Does ClassicLRU or v003 degrade on other workload distributions, cache sizes, or multi-core contention scenarios?** Both cycles 0001–0004 tested only Zipfian access patterns and single-threaded execution. Robustness across uniform, exponential, or phase-changing access distributions, larger caches, or concurrent workloads is unexplored and could reveal failure modes.

- **Why is ClassicLRU's vector manipulation on 16-item buckets so fast?** Cycle 0002 raised this unanswered: O(n) remove-and-append on small vectors beats O(1) timestamp updates. Profiling (cache line coherency, branch prediction, loop unrolling, memory layout) could reveal whether this is an accident of current hardware or a fundamental lesson for cache design.

## Per-policy notes

- **ClassicLRU (baseline)**: Hit-rate stable at 0.6611 across all geometry and policy variants. Latency performance is exceptionally hard to beat—attempts to improve it through policy complexity consistently fail. The implementation appears finely tuned for small-vector (16-item) bucket scans, with excellent cache locality and compiler optimization. Its dominance is not primarily algorithmic but implementational.

- **LFU (cycle 0001)**: Frequency tracking added 45% latency overhead without improving hit-rate (0.6568, slightly worse than 0.6611). The Zipfian workload is sufficiently concentrated that recency alone captures the access pattern; explicit frequency awareness yields no benefit.

- **FIFO (cycles 0002, 0004)**: Zero-bookkeeping FIFO maintained ClassicLRU's hit-rate (0.6611) on baseline geometry in cycle 0002, proving that the Zipfian workload's access concentration does not require recency tracking. However, cycle 0004's attempt to pair FIFO with victim-selection optimization on wide buckets regressed latency 2–4×. The zero-bookkeeping promise was not realized; a second attempt with simpler victim selection or per-hit bookkeeping might succeed.

- **TimestampLRU (cycle 0002)**: O(1) timestamp updates on each hit replaced ClassicLRU's O(n) vector removal-and-append. Despite asymptotic superiority, latency regressed ~40%, and hit-rate held at 0.6611. The micro-architectural cost of timestamp updates (memory write, cache coherency) outweighed the vector operation savings. In cycle 0004, using timestamps for victim selection (O(n) bucket scan to find min-timestamp) regressed 2–4×.

- **Segment-based LRU (cycle 0004)**: Per-operation segment metadata maintenance caused ~4× latency regression despite bounding the eviction scan. Adding complexity to an already-rare operation (eviction) while burdening every hit with metadata updates is a losing trade-off.

- **Bucket geometry**: Narrower buckets (256×8 in cycle 0003) backfired despite reducing scan depth, regressing both hit-rate and latency by ~45%. Hash distribution overhead dominates. Wider buckets (64×32, 32×64 in cycles 0003–0004) improve hit-rate progressively (+0.24%, +0.0005%), but without micro-optimization cause tail-latency spikes. ClassicLRU's baseline geometry (128×16) remains the safest choice unless paired with implementation-level optimizations (v003's early-exit and cache-line alignment).
