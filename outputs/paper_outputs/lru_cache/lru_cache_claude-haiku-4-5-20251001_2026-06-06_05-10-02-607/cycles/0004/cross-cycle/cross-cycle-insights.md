# Cross-Cycle Insights

## Recurring wins

- **Structural data structure solutions are more reliable than algorithmic tweaks**: Bounded search windows (Cycle 0002 V001) and frequency bucket indexing (Cycle 0002 V002) both consistently eliminated O(N) pathological eviction latency spikes, whereas pure strategy changes (frequency ranking depth, sampling schemes) added overhead without proportional gain. The implementation structure (how candidates are stored and accessed) matters more than the eviction policy itself.

- **Hit rate and tail latency are decoupled when the right data structure is used**: Cycles 0001–0002 initially suggested an unavoidable trade-off, but Cycle 0002 V002 proved both metrics can improve simultaneously if eviction scanning is efficient. The perceived trade-off was an artifact of inefficient linear scanning, not fundamental conflict.

- **Workload-aware adaptive modes beat uniform strategies**: Cycle 0003 V003 achieved 34% get p99 improvement by selectively activating expensive frequency bucketing only under high saturation, outperforming V001 and V002 which paid the cost uniformly regardless of cache pressure. This demonstrates that strategy selection must be conditional on runtime state.

- **Bounded-window sequential scanning remains latency-superior to random sampling**: Despite multiple attempts to replace deterministic scanning with random sampling (Cycle 0004 V003) or deferral (Cycle 0004 V001–V002), the bounded-window baseline's predictability and avoidance of per-item metadata overhead consistently outperformed alternatives on latency metrics.

## Recurring failures

- **Linear eviction scans invite O(N) latency spikes unless bounded**: Cycles 0001–0003 repeatedly showed that searching the full eviction list for a candidate, even with selective marks (high frequency, resurrection count), causes dramatic tail latency variance. Only structural bounding (Cycle 0002 onwards) fixed this.

- **Metadata overhead taxes all caches, utilized or not**: Every variant that added tracking—frequency counters, zone markers, resurrection maps, deferred-work queues—increased per-item or per-cache memory footprint. On underutilized caches, this cost is paid without corresponding hit rate gain (Cycles 0003–0004 showed hit rate stability despite metadata variations).

- **Deferral shifts cost between operations rather than eliminating it**: Cycle 0003 V001 and Cycle 0004 V002 both deferred expensive work from the critical path (get) to less-frequent operations (put/eviction). Get latency improved, but put regressed 26–56% (Cycle 0004 V002) or spiked 64% (Cycle 0003 V001). The underlying work must be executed; deferral merely changes when the cost is paid.

- **Per-operation frequency tracking and sampling bookkeeping add more overhead than the logic being optimized**: Cycle 0004's sampling (V001, periodic modulo checks) and per-item counters (V003) both regressed on light workloads where decisions are rare. The bookkeeping cost (counter increments, mode checks, memory barriers) exceeded the cost of the lightweight eviction logic being optimized.

- **Frequency ranking granularity yields diminishing returns within bounded windows**: Cycle 0003 V002's multi-level frequency ranking within a bounded window improved hit rate marginally but hurt latency universally (+38% put p99). A binary threshold (high-frequency vs. low-frequency) within a small scan window already captures most selectivity.

## Open questions

- **Can V003's adaptive bucketing (Cycle 0003) scale beyond the underutilized workload tested?** All cycles used a relatively light workload; saturation thresholds that trigger expensive bucketing never fired in Cycle 0004. Does adaptive mode-switching justify its 72-byte metadata tax when saturation is sustained or workload patterns shift rapidly?

- **Is the workload's Zipfian distribution and light utilization masking optimization opportunities?** All variants maintained hit rate at ~0.6904 despite radically different eviction logic. Does this indicate LRU within a bounded window already captures the hot set, or would broader access patterns or sustained high saturation reveal opportunities for more sophisticated policies?

- **Can any deferral or batching strategy improve multiple operation types simultaneously, or is asymmetric cost-shifting fundamental?** Cycles 0003–0004 show deferral reliably trades get for put. Is there a buffering scheme that diffuses the deferred work across operations, or does batching inherently concentrate it?

- **What fundamentally different eviction strategies exist beyond frequency, recency, and resurrection tracking?** All cycles explored variants of frequency-awareness or window-bounding. Could access-pattern prediction, time-window tracking, or sketch-based approximate frequency offer better latency-hit-rate trade-offs?

## Per-policy notes

- **Zone partitioning and bounded windows (V001 line)**: Cycle 0001 V001 (spatial zones) failed to improve hit rate but improved put latency via simpler decisions. Cycle 0002 V001 (bounded search) succeeded by constraining structural complexity to O(capacity/4), avoiding metadata overhead and achieving 3.2× put p99 gain. Cycle 0003 V001 (lazy-batch updates) attempted to defer work but shifted cost to put spikes. Cycle 0004 V001 (periodic sampling) added bookkeeping overhead that hurt light workloads. **Lesson**: Simple structural bounds win; deferral and sampling add complexity without net gain on this workload.

- **Frequency and bucket indexing (V002 line)**: Cycle 0001 V002 (frequency-recency hybrid) achieved best hit rate (0.6904) but spiked put p99 due to linear scanning. Cycle 0002 V002 (bucket indexing) fixed the scan bottleneck and won on both hit rate and put p99, but added measurable normal-case overhead (get p99: 67→147 ns). Cycle 0003 V002 (multi-level ranking) added decision complexity without hit rate gain and regressed latency. Cycle 0004 V002 (deferred promotions) improved get p99 (23% gain) but regressed put severely (26–56%). **Lesson**: Bucket indexing is the most robust frequency strategy; deeper ranking and deferral add complexity without proportional gains.

- **Adaptive and tracking strategies (V003 line)**: Cycle 0001 V003 (resurrection count) showed modest hit rate gains with high latency variance. Cycle 0002 V003 (zone+frequency hybrid) failed due to expensive scanning contradicting its stated goal. Cycle 0003 V003 (adaptive bucketing) emerged as a winner—34% get p99 improvement via conditional activation. Cycle 0004 V003 (random sampling + per-item frequency) catastrophically regressed put p99 (2.8×) despite 1.3% hit rate gain. **Lesson**: Adaptive mode-switching (Cycle 0003 V003) is the only V003 variant that succeeded; per-item tracking and random sampling underperform deterministic bounded scanning.
