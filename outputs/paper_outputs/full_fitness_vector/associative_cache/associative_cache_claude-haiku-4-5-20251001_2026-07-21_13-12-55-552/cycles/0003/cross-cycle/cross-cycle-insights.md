# Cross-Cycle Insights

## Recurring wins

- **ClassicLRU remains unbeaten**: Across all three cycles, the baseline has not been displaced by any variant despite testing six distinct policy and geometry changes. This suggests the baseline is not merely correct but finely tuned for the current workload and hardware.
- **FIFO viability**: When correctly specified, FIFO maintained 0.6611 hit-rate on a Zipfian workload without explicit recency bookkeeping, proving that hit-rate is not tightly coupled to per-hit tracking overhead.
- **Geometry-hit-rate trade-off exists**: Wider buckets consistently nudged hit-rate upward (best: +0.24%), confirming that reduced hash collisions or improved cache density do correlate with hit-rate gains.

## Recurring failures

- **Policy swaps degrade latency uniformly**: LFU, TimestampLRU, and Swap variants regressed latency by 40%–45% or worse despite aiming at per-operation cost reduction or hit-rate improvement. No policy change has recovered or matched baseline latency.
- **Theory-practice inversion on asymptotic complexity**: Swap (O(1) removal), TimestampLRU (O(1) bookkeeping), and FIFO (zero per-hit cost) each hypothesized lower asymptotic cost than ClassicLRU's O(n) vector ops, yet all measured slower in practice. This pattern repeats across cycles, implicating cache micro-architecture and implementation overhead as the dominant factors.
- **Narrower geometry backfired**: Reducing bucket size (16→8) failed despite the theory that shallower linear scans would reduce per-operation cost. Hash-table lookup and memory-layout overhead across more buckets outweighed any scan-depth benefit.
- **Wider geometry saturates**: Hit-rate gains from wider buckets (16→32→64) are diminishing, and tail-latency penalties accelerate sharply, suggesting an inflection point where scan-depth overhead dominates.

## Open questions

- **Why is ClassicLRU's O(n) so fast?**: Multiple O(1) alternatives measured slower. Is the answer cache-line coherency, branch prediction stability, or compiler loop optimization of small vector operations? Profiling and code inspection are needed.
- **Can FIFO decouple latency from bucket width?**: Cycle 0003's FIFO variant failed to execute. If corrected, would zero per-hit bookkeeping flatten latency costs while wider buckets improve hit-rate, or is there a fundamental trade-off?
- **Is tail-latency spike inherent or fixable?**: The sharp p99 degradation in 32×64 geometry may be an artifact of worst-case bucket scans or implementation details rather than a fundamental limit. Micro-architectural analysis could clarify.
- **Does ClassicLRU break outside Zipfian access patterns?**: All cycles tested the same Zipfian workload. Robustness across uniform, skewed, temporal, or multi-core contention scenarios remains untested.

## Per-policy notes

- **ClassicLRU**: Baseline. Remains unbeaten despite six targeted variants. Hit-rate stable at 0.6611; latency is the performance bar.
- **LFU**: Frequency awareness yielded no hit-rate improvement (0.6568), suggesting Zipfian recency alone captures the access pattern. Latency regressed ~45%.
- **TimestampLRU**: O(1) timestamp updates hypothesized to beat ClassicLRU's O(n) vector ops. Hit-rate held (0.6611); latency regressed ~40–45%, inverting the asymptotic prediction.
- **FIFO**: Proved capable of 0.6611 hit-rate without per-hit recency tracking. Cycle 0003 variant failed to execute; correct implementation on wider buckets remains untested.
- **Swap**: Pursued O(1) removal via pointer-swap chains. Hit-rate worsened (0.6568); latency regressed by 3×. Constant-factor overhead dominated asymptotic advantage.
- **Bucket geometries**: 16×128 (baseline) is locally optimal for latency. 256×8 (narrower) regressed both metrics. 64×32 (wider) improved hit-rate (+0.24%) at ~45% latency cost. 32×64 (even wider) saturated hit-rate gain (+0.0005%) while tail latency spiked, marking an inflection point.
