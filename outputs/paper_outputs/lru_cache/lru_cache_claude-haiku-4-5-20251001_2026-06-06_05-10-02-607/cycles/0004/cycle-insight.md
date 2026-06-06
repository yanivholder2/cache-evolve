# Cycle 0004 Insight

## Cohort outcome

Three optimization strategies targeting the Cycle 0003 adaptive bucketing baseline were tested: periodic sampling to reduce decision overhead (V001), deferred frequency promotions from get to put (V002), and random sampling with per-item frequency tracking (V003). Only V002 improved on the baseline, gaining 23% in get p99 latency; V001 and V003 both regressed sharply across latency metrics. V002 won by trading put latency stability for get performance—a clean win on one axis at the cost of another, unlike V001 and V003 which hurt without corresponding gains.

## Patterns

- **Light workload overhead is a trap for sampling-based optimizations**: V001's periodic sampling (every 16 operations) intended to skip decision logic on fast-path operations, but the counter increment and modulo check on every operation cost more than the lightweight decision logic being executed. On an underutilized cache where decisions are rare (mode transitions never triggered), the "optimization" was pure overhead. This teaches that decision-deferral strategies require high contention or frequent re-evaluation to justify their bookkeeping cost.

- **Deferral reliably shifts cost between operations without eliminating it**: V002 deferred frequency bucket updates from get() to put(), reducing get p99 by 23% (147→114 ns). However, put p50 regressed 26% (70→88 ns) and put p99 regressed 56% (190→297 ns), especially under read-heavy patterns where promotions batch up before a put. The work didn't disappear; it moved. This confirms the pattern identified in Cycle 0003 and reinforces that deferral is a trade-off strategy, not a solution.

- **Per-operation frequency tracking is incompatible with latency predictability**: V003 added a frequency counter to each node, improved hit rate by 1.3%, but catastrophically regressed put p99 by 2.8× (135→383 ns). The counter increment and memory barrier effects on every access dominate the latency profile, making random sampling of low-frequency candidates unpredictable and expensive. Even when sampling reduces worst-case scan length, the per-item tracking overhead makes the fast path slower.

- **Bounded-window sequential scanning (Cycles 0002–0003 baseline) remains latency-superior to random sampling**: V003's attempt to eliminate sequential scanning via random sampling of 32 candidates paradoxically produced worse tail latency than the deterministic bounded window it replaced. Random sampling plus frequency tracking introduces variability and forces counter overhead on every access; the bounded window's sequential nature is predictable and avoids per-item metadata.

- **Hit rate stability across all variants confirms the bounded-window foundation is sufficient**: V002 maintained hit rate at 0.6904 (unchanged from V003 baseline) despite moving frequency updates off the fast path. V003 gained 1.3% hit rate (0.6904→0.6987) but at catastrophic latency cost. The Cycle 0003 adaptive bucketing's selectivity within the bounded window is already near-optimal for this workload; further sophistication in eviction selection does not move hit rate.

## Open questions

- Is V002's put latency regression (26–56%) acceptable in production for read-heavy workloads? The gain is substantial (23% get p99), but whether the asymmetric trade-off justifies a variant depends on real workload composition.

- Why does the underutilized workload consistently penalize frequency-aware strategies? Is the workload's Zipfian skew strong enough that simple LRU within a bounded window already captures the hot set, making frequency tracking's overhead wasted? Or would frequency tracking shine under sustained high saturation or broader access distributions?

- Can any deferred-work strategy be designed to avoid the operation-shifting trade-off? V002's get gain came at put's expense. Is there a buffering or batching scheme that improves both, or is asymmetric cost-shifting fundamental to deferral?

## Continuation

V002 emerges as a viable optimization for read-dominated workloads where get p99 latency is the priority metric and put operations are less frequent or less latency-sensitive. Its 23% improvement in get p99 comes at the cost of 26–56% put regression, making it a workload-specific choice rather than a universal upgrade. However, the cohort's broader pattern reveals diminishing returns: Cycle 0003's adaptive bucketing baseline is already well-tuned for this cache's typical operating point (underutilized, Zipfian skew). Attempts to optimize further—via sampling, deferral, or frequency tracking—either add overhead that hurts light-workload cases or shift cost between operations without real net gain. 

The next cycle should either: (a) stress-test V002 on longer traces and read-heavy production patterns to validate that the put regression is acceptable and get improvement holds, (b) explore fundamentally different eviction strategies (e.g., time-window tracking, access-pattern predictors) that don't rely on frequency counters or linear scanning, or (c) investigate whether the workload's underutilization is masking optimization opportunities that would emerge under sustained high saturation or adversarial access patterns. If V002's trade-off is deemed acceptable for the target use case, it should be integrated into Cycle 0005's baseline; otherwise, Cycle 0003's v003 remains the best-balanced design.
