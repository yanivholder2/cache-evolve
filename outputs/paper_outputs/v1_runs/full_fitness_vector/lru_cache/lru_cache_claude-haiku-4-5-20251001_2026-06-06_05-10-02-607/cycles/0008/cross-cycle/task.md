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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Three strategies for optimizing Cycle 0004's deferred-promotion baseline were tested: lazy bucket allocation (V001), incremental queue draining during get operations (V002), and saturation-conditional deferral (V003). Only V002 achieved a net win—it reduced put p99 spikes by 42% (297→173 ns) and put p50 by 2% by distributing deferred work across both operations. V001 failed due to pathological mode-transition overhead from dynamic allocation near saturation boundaries, causing get p99 to spike 40×. V003, while architecturally sound, remained unused on the tested workload: the cache never reached its 80% saturation threshold, so it defaulted to immediate frequency updates that negated V002's optimization.

## Patterns

- **Dynamic allocation for mode-switching is costlier than pre-allocation**: V001's lazy bucket initialization via `Option<Box<>>` was intended to eliminate the 72-byte permanent metadata tax on underutilized caches. Instead, it introduced repeated allocation/deallocation cycles when the workload oscillated near saturation boundaries (60% hysteresis threshold), creating catastrophic get p99 latency spikes (40×). Pre-allocated structures, while permanently present, avoid the pointer indirection and allocation overhead that V001 paid on every mode transition.

- **Incremental batching successfully amortizes deferred work across operations**: V002's approach of checking the promotion queue during get() and draining a small batch (e.g., 4 items) when queue depth exceeded a threshold (e.g., 8 items) effectively split the burden. Put p99 dropped from 297 ns (all-at-once flush) to 173 ns (incremental drains during gets), reducing the peak spike while accepting a 32% get p99 regression (114→151 ns). This is a cleaner trade-off than Cycle 0004's V002 (56% put regression from pure deferral) and demonstrates that amortization technique matters more than deferral strategy alone.

- **Saturation-based condition logic only wins when conditions actually fire**: V003 added a `deferral_enabled` flag and 80% saturation threshold to activate V002's deferred-promotion queue only under high load. On the underutilized test workload, saturation never exceeded 60%, so the variant spent its entire run in the immediate-update branch—paying the 8-byte metadata overhead and decision logic while receiving none of the deferral benefits. The architecture is sound but requires workload validation; pre-computed thresholds become liabilities if the target domain is lighter than expected.

- **Hysteresis window width matters for mode-transition costs**: V001 used a 60% disable threshold (vs. Cycle 0003's v003 which used 75%), making the saturation boundary narrower. This aggravated oscillation near the threshold, triggering frequent allocation cycles. The tighter hysteresis region created more mode transitions, directly driving the catastrophic latency regression. Trade-offs in mode-switching boundaries ripple through the cost of the switch operation itself.

- **Work distribution across operations beats structural optimization when workload utilization is light**: All three variants attempted to optimize metadata overhead or batching, but V002's straightforward distribution mechanism outperformed both memory optimization (V001) and conditional activation (V003) on the actual workload. The lesson: smart work distribution at realistic load points trumps structural elegance or conditional complexity.

## Open questions

- Will V002's get p99 regression (32%, 114→151 ns) prove acceptable for the target use case when put p99 improves 42%? The trade-off depends on read vs. write ratio in production.

- Can V003's saturation thresholds be tuned or can the workload be stress-tested with longer traces where sustained high-saturation phases occur, validating whether conditional deferral becomes beneficial?

- Can V002's incremental draining thresholds (queue depth, drain batch size) be further optimized to reduce the get p99 regression without sacrificing the put p99 gain?

- Did V001's pathological behavior stem solely from allocation costs, or did the tighter 60% hysteresis boundary itself cause excessive oscillation that would be present in any dynamic-allocation scheme?

## Continuation

V002 emerges as the clear winner: incremental queue draining is a practical amortization technique that reduces the operation-shifting trade-off identified in earlier cycles. By spreading promotion work across both get and put, it achieves meaningful put p99 improvement (297→173 ns) while accepting measured get p99 cost. V001 proved that memory optimization via lazy allocation backfires when mode transitions are frequent or near-boundary oscillation occurs; the allocation/deallocation overhead dominates any memory savings. V003 was architecturally sound but failed because the tested workload never saturates—a cautionary tale for threshold-based designs: validate that conditions fire in the real domain before complexity investment.

Next cycle should: (a) integrate V002's incremental draining as the new baseline and explore threshold tuning (queue depth, batch drain size, and drain frequency) to further improve get p99 without surrendering put gains, (b) stress-test V003 on longer traces and synthetic workloads with sustained high-saturation phases to determine if conditional deferral actually wins when its thresholds trigger, and (c) investigate whether V001's pathological behavior can be salvaged by adjusting hysteresis boundaries or switching to fixed allocation rather than lazy initialization.


### Cycle 0006

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


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

Three optimization strategies targeting the Cycle 0006 baseline were tested: batched queue-flush coalescing to reduce put p99 cost without reintroducing checks on the hot path (V001), pure FIFO eviction with a larger window to eliminate frequency-tracking overhead (V002), and saturation-aware mode switching to minimize overhead at light load while preserving deferred-promotion gains at high saturation (V003). No variant achieved a net win without regressions, but V001 demonstrated a breakthrough: simultaneous improvement of both get p99 (114→66 ns, 42%) and put p99 (297→149 ns, 50%), at the cost of a 20% get p50 regression. V002 validated that hit rate is window-size-dependent and frequency tracking eliminable, but latency improvements proved marginal. V003 catastrophically failed when the saturation threshold never materialized, breaking LRU correctness and causing hit rate to plummet 9.4%.

## Patterns

- **Algorithmic queue-flush optimization achieves balanced tail-latency improvement for the first time**: V001's two-batch swap strategy and fixed-size batch processing (16 items per batch) improved both get and put p99 simultaneously, breaking the operation-shifting trade-off pattern from Cycles 0004–0006. The key insight is that coalescing consecutive puts to share batch-flush overhead reduces peak work-per-operation more efficiently than incremental amortization (which spreads overhead across both operations) or complete deferral (which concentrates it on put). This validates that deferred-work trade-offs are tunable through algorithmic optimization of the flush mechanism, not inherent to deferral itself.

- **The cost of batch-swap bookkeeping on the hot path is measurable but bounded**: V001's 20% get p50 regression (35→42 ns) reflects the overhead of managing two batch vectors and swapping logic on every get, even those following recent puts. However, this overhead is predictable and affects median latency, not tail (p99 improved 42%). The trade-off is more favorable than V002's (Cycle 0006) V001, where drain-trigger checks degraded tail latency; V001 here prioritizes median consistency for magnitude tail-latency gains.

- **Hit rate is definitively window-size-dependent; frequency tracking is eliminable overhead**: V002 confirmed that pure FIFO with a larger window (min(capacity/2, 512)) maintains hit rate (0.6904, unchanged from parent) while removing all per-item frequency metadata. This validates the cross-cycle hypothesis that Zipfian skew is captured by window size alone. However, the marginal latency improvements (put p50 +3%, all others flat or regressed) reveal that frequency-tracking overhead is not a latency bottleneck on this light-load workload—simpler algorithms do not significantly improve latency even when overhead is eliminated.

- **Deferring correctness-critical operations without universal activation causes severe failures**: V003's saturation-aware mode switching aimed to use simple FIFO at low saturation and deferred-promotion at high saturation. However, the tested workload never saturated above 70%, leaving the cache in low-saturation mode with deferred promotions but no promotion feedback, breaking LRU ordering. The result was a 9.4% hit-rate drop (0.6904→0.6255), 71% get p50 regression (34→58 ns), and 43% get p99 regression (97→139 ns). This teaches that conditional logic protecting correctness-critical operations must either activate universally or be designed to preserve baseline semantics when inactive—deferring promotions without threshold-based feedback invalidates the LRU invariant.

- **The bounded-window baseline is Pareto-complete for light workloads at this latency granularity**: All three variants showed either no net improvement or clear regressions despite addressing different optimization axes (queue flushing, metadata overhead, conditional activation). Only V001's tail-latency improvements on p99 are significant, but the p50 regression prevents it from being a clear winner. This suggests the baseline is well-tuned for the tested workload's light utilization, and further gains require either higher saturation (where deferred-promotion cost justifies its p99 benefit) or fundamentally different optimization targets (e.g., cache-line alignment, memory access patterns).

- **Tail latency is now the dominant optimization axis; hit rate is saturated**: Across Cycles 0004–0007, hit rate has plateaued (~0.66–0.69 regardless of eviction strategy), while latency optimization has become increasingly focused on p99 and p50 distributions. V001's value proposition is entirely on p99 reduction; hit rate did not change. This reflects a mature caching implementation where selectivity is no longer the bottleneck and latency predictability becomes the primary concern.

## Open questions

- Can V001's get p50 regression (20%) be reduced through threshold tuning (e.g., batch size, swap frequency, drain conditions) without sacrificing the p99 improvements? The queue-swap mechanism's bookkeeping cost is real; does it scale favorably if batch size increases or swaps become less frequent?

- Does V001's balanced improvement (both p99s better) persist under sustained high saturation or write-heavy workloads, or is it an artifact of the light-load trace where batch accumulation is predictable?

- Why are V002's latency improvements marginal despite eliminating frequency-tracking overhead entirely? Does this suggest that on this workload, memory hierarchy effects or branching cost more than per-operation bookkeeping, making frequency-tracking elimination a red herring?

- Can V003's saturation-aware mode switching be salvaged by adjusting the activation threshold or by using a mode-conditional semantics that preserves LRU promotion feedback in low-saturation mode (e.g., asynchronous promotions without blocking)?

## Continuation

V001 emerges as the most promising design in this cycle, demonstrating for the first time that queue-flush algorithmic optimization can achieve balanced tail-latency improvement without the operation-shifting trade-offs that plagued Cycles 0004–0006. The 20% get p50 regression is a measurable but bounded cost; threshold tuning on batch size and swap frequency should be explored to reduce this overhead while preserving the substantial p99 gains. V002 validates that hit rate plateaus at window size and frequency tracking is eliminable, but the marginal latency improvements suggest frequency-tracking overhead is not a bottleneck on this light-load workload. V003 failed decisively and teaches a critical lesson: deferring correctness-critical operations (LRU promotions) based on unmet saturation assumptions invalidates cache semantics; conditional activation must either fire universally or preserve baseline behavior when inactive.

Next cycle should: (a) integrate V001's batched queue-flush mechanism as a new baseline and systematically tune batch size, queue-swap frequency, and drain thresholds to minimize get p50 regression while preserving p99 gains, (b) stress-test V001 on longer traces, higher saturation levels, and write-heavy workloads to determine whether the balanced improvement holds or whether put p99 cost increases under sustained load, (c) investigate whether V001's queue optimization can be applied to V002's simpler FIFO strategy to test whether batched processing improves tail latency even without deferred promotions, and (d) revisit saturation-aware mode switching with either corrected activation thresholds (via trace analysis) or mode-conditional semantics that preserve LRU correctness at all saturation levels.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Three approaches targeting Cycle 0007's batched queue-flush baseline were tested: expanding batching infrastructure with mode-aware deferred promotion (V001), decoupling batched queue-flush from frequency tracking via pure FIFO (V002), and lowering saturation thresholds to maximize deferred-promotion activation (V003). No variant achieved a net win. V001 regressed both get and put latencies due to unused batching overhead in simple mode. V002 failed when frequency tracking was removed, exposing that batched queue-flush optimization was context-dependent. V003 achieved its narrow goal (36% get p99 improvement) but the per-operation saturation check introduced uniform overhead that caused 31% get p50 regression, negating broader benefit.

## Patterns

- **Deferred-work infrastructure pays off only when the expensive path is hot**: V001 added Vec allocations, threshold checks, and batch-swap logic expecting to amortize overhead through frequent frequency bucket updates at high saturation. The workload never saturates, staying in simple mode throughout, making all batching infrastructure pure overhead—constant cost on every operation despite being completely idle. This reinforces the pattern from Cycle 0007: deferred-work optimization is only valuable when the cost being deferred is actually incurred frequently.

- **Algorithmic optimizations are not orthogonal across contexts**: V002 attempted to separate batched queue-flush (which succeeded in Cycle 0007) from frequency tracking by switching to pure FIFO with window-based selective promotion. The result was catastrophic: get p99 regressed 203% (83→252 ns), hit rate dropped 4.8%, and metadata grew despite removing per-node frequency storage. This teaches that optimizations appearing independent may be tightly coupled to their original context—batching overhead was justified by improved eviction decisions in the frequency-tracking context; without that benefit, the batching cost dominates.

- **Conditional logic in hot paths must be nearly free or asynchronous**: V003 achieved its stated goal (36% get p99 improvement via frequent deferred-promotion activation at 45% saturation) but the per-operation saturation check introduces uniform overhead (division, branch) that causes 31% get p50 regression. The tail-latency win is offset by broader median-latency cost. This demonstrates that condition checking in critical paths requires extreme efficiency or asynchronous tracking to avoid negating the benefit of the condition itself.

- **Workload saturation profile is a first-order constraint on optimization space**: All three variants implicitly assumed different workload behavior than reality. V001 assumed sustained high saturation (never occurred). V002 assumed frequency tracking was a latency bottleneck (it wasn't; the bottleneck was batching overhead, which relocated to window-checking). V003's 45% threshold fires frequently as designed, but the workload's light utilization means expensive operations (frequency updates) are not on the critical path, limiting deferral benefit to tail latency. Optimization strategies must be validated against actual workload characteristics before scaling.

- **Hit rate is now determined solely by eviction strategy and window size**: V001 maintains parent's hit rate (0.6904). V002 degrades 4.8% (0.6904→0.6577) when frequency tracking is removed, confirming window size cannot fully capture the selectivity that frequency provides. V003 maintains hit rate despite deferred promotions, proving LRU correctness is preserved. Hit rate is independent of batching, saturation checks, and deferral mechanics—it depends entirely on eviction algorithm and parameters.

## Open questions

- Can V003's saturation check overhead be eliminated by computing it asynchronously (sampled, tracked via counter update, updated at allocation boundary) without sacrificing the 36% get p99 improvement?

- Does V001's failure stem from the batching infrastructure itself, or from the assumption that simple mode dominates? Would more aggressive thresholds or smaller batch sizes transform overhead into benefit?

- Can V003 and V001 be combined—using conditional deferred promotion only when saturation fires, applying batching only when that condition is active? Would this eliminate both V003's per-operation check cost and V001's idle-queue overhead simultaneously?

- Is V002's 4.8% hit-rate degradation fundamental to FIFO+window strategies, or an artifact of the specific window-checking cost? Does removal of frequency tracking always degrade selectivity on this workload?

## Continuation

This cycle reveals a critical lesson: algorithmic optimizations are context-dependent, and workload saturation profile is a first-order constraint that must be validated before complex infrastructure investment. V001 failed because deferred-batching overhead only amortizes when the deferred work is frequent; this workload doesn't saturate. V002's catastrophic failure demonstrates that optimizations cannot be cleanly decoupled across contexts—batching worked with frequency tracking because better eviction decisions justified the cost; without them, the cost is pure overhead. V003 achieved its narrow goal but reveals that hot-path condition checking introduces uniform overhead that negates benefit unless nearly free.

Next cycle should: (a) validate whether Cycle 0007's batched queue-flush baseline remains optimal by testing V003's saturation check as a runtime decision (not infrastructure-driving) to see if conditional deferral enables the parent's batched design without V003's overhead, (b) measure actual saturation distribution across the test trace to establish whether higher saturation levels ever occur and whether optimization landscape changes at those points, (c) investigate whether V003's saturation check can be computed asynchronously or amortized to recover the 36% get p99 gain without the 31% get p50 cost, and (d) confirm whether frequency tracking is fundamentally non-eliminable on this workload by testing whether any frequency-free eviction strategy can match hit rate.


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
