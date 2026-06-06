# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three strategies targeting hit rate improvements via different eviction signals were tested: spatial zone partitioning (V001), frequency-recency hybrid (V002), and resurrection count tracking (V003). V002 achieved the strongest hit rate and get latency gains, while V001 improved put performance through simpler eviction decisions. V003 showed modest hit rate gains but with higher latency variance.

## Patterns

- **Hit rate-latency trade-off**: Both V002 and V003 improved hit rate by making eviction decisions more selective, but this selectivity required O(N) worst-case scans through the eviction list, causing dramatic tail latency spikes (put p99: 20× for V002, 2× for V003). V001 avoided this by pre-partitioning but saw no hit rate benefit.

- **Metadata cost is universal**: All three variants added tracking overhead—zone markers, frequency counters, or eviction history maps. This increases per-item or per-cache memory footprint consistently, a tax paid even by underutilized caches.

- **Get and put performance diverge by strategy**: V002's frequency tracking improved both get latency (faster spatial locality from retaining hot items) and put median (less churn), yet spiked put tail. V003 hurt both get and put tails through metadata contention. V001 improved put but penalized get with promotion logic.

- **Eviction search efficiency is the bottleneck**: V002 and V003 both suffer from linear scans to find an evictable node when most candidates have protective marks (high frequency or resurrection count). The simpler zone-based eviction in V001 avoided this, suggesting that structural partitioning may be more latency-predictable than selective scanning.

## Open questions

- Can hit rate improvements be achieved without full linear eviction scans? (e.g., bucketed frequency lists, approximate resurrection tracking)
- Does V002's frequency decay mechanism, if better tuned, actually suppress the observed put p99 spike in production workloads?
- How sensitive are V002 and V003 to cache saturation and working-set stability? Do their metadata costs exceed benefits on underutilized caches?

## Continuation

The cycle reveals a fundamental tension: selective eviction improves hit rate but invites latency unpredictability, while structural partitioning is latency-friendly but may miss hit rate gains without per-item intelligence. V002 stands out for improving hit rate *and* get performance; its put p99 spike merits targeted mitigation (e.g., eviction bucket structures or heuristic pruning) rather than abandonment. V003's marginal gains do not justify its overhead. Next cycle should pursue either: (a) efficiency optimizations for V002's eviction scan (approximate frequency or bucketed lists), (b) V002 frequency decay tuning to empirically validate tail latency suppression, or (c) hybrid approaches that combine spatial robustness (V001) with selective intelligence (V002) under latency constraints.


### Cycle 0002

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


### Cycle 0003

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


### Cycle 0004

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


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
