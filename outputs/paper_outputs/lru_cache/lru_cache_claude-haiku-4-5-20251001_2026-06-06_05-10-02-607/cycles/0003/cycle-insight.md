# Cycle 0003 Insight

## Cohort outcome

Three strategies targeting the latency-hit-rate trade-off identified in Cycle 0002 were tested: lazy-batch bucket updates (V001), multi-level frequency ranking within bounded windows (V002), and adaptive bucketing tied to saturation thresholds (V003). All three maintained hit rate at 0.6904, but diverged sharply on latency: V003 achieved the strongest improvement (34% reduction in get p99) by selectively activating frequency bucketing only under high saturation; V001 successfully improved get latency but shifted the cost to eviction-time put spikes; V002 found that finer-grained frequency ranking within the bounded window incurs consistent latency overhead across all operation types without proportional hit rate gain.

## Patterns

- **Deferred work concentrates overhead rather than smoothing it**: V001's strategy of deferring bucket reorganization from get to batch flush points did reduce get p99 (147→140 ns), but the deferred work was concentrated into put operations at eviction boundaries, causing put p99 to spike 64% (190→311 ns). This shows that the underlying work must be executed; deferral merely shifts when the cost is paid, creating variance in a different operation rather than eliminating it.

- **Workload-aware mode selection outperforms uniform strategy**: V003's adaptive approach (simple mode under 75% saturation, bucketing mode above) achieved superior latency on an underutilized workload by spending most operations in the lightweight simple mode, cutting get p99 by 34%. In contrast, V001 and V002 applied expensive logic uniformly, paying the cost regardless of cache pressure. This suggests that strategy selection must be conditional on workload state, not fixed at design time.

- **Frequency ranking granularity has diminishing returns within bounded windows**: V002's attempt to rank candidates by multi-level frequency within the bounded search window improved hit rate marginally but hurt latency universally (+38% put p99). This indicates that V001's binary threshold (freq ≤ 1) may already capture most selectivity benefit within a small scan window; finer granularity adds decision complexity without proportional gain.

- **Metadata overhead is a fixed cost regardless of utilization**: V001 and V003 both increased the LruCache struct size (V001: +32 bytes for pending-update queue; V003: +72 bytes for dual-mode state), and this cost applies even to underutilized caches that never exercise the expensive code paths. V003's larger tax is offset by its conditional activation of expensive logic, but the trade-off requires workload validation.

- **Hit rate stability across all variants suggests bounded-window sufficiency**: All three variants maintained 0.6904 hit rate despite fundamentally different eviction strategies, indicating that the bounded-window search established in Cycle 0002 is a stable foundation. Variations in how candidates are selected within the window (binary threshold, multi-level ranking, or adaptive bucketing) do not substantially move hit rate on this workload.

## Open questions

- Will V003's adaptive bucketing prove cost-effective on longer workloads or those that frequently cross saturation thresholds? The tested workload remained underutilized, never triggering high-saturation mode, so the value of mode-switching infrastructure remains unvalidated.

- Can the decision overhead in V001 and V003 (checking pending-update queues, saturation tracking, mode-transition logic) be reduced, or does it inherently increase median latencies?

- Does V002's multi-level frequency ranking improve hit rate on non-Zipfian workloads with broader frequency distributions, or does the latency cost outweigh gains universally?

## Continuation

V003 emerges as the most balanced design: it achieves V001's latency improvements on normal operating conditions while preserving V002's bucketing capability for high-saturation scenarios where hit rate pressure is greatest. The 72-byte metadata overhead is acceptable if mode-switching triggers remain rare. The next cycle should: (a) stress-test V003 on longer traces and workloads that cross saturation thresholds to validate that mode-switching overhead and structure-rebuild costs are justified, (b) measure whether V003's saturation-driven activation actually converges to higher hit rate under sustained pressure, or if the bounded-window baseline already captures sufficient selectivity, and (c) consider optimizing V003's mode-transition logic to reduce the constant overhead of saturation and hit-rate tracking (e.g., periodic sampling instead of per-operation checks). If V003's adaptive approach holds under stress, it should become the new baseline for Cycle 0004.
