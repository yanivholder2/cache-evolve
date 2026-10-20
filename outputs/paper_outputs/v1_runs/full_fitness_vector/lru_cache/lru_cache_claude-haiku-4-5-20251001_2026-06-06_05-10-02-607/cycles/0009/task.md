# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/lru_cache/lru_cache_claude-haiku-4-5-20251001_2026-06-06_05-10-02-607/cycles/0009

## Winners

(none)

## Variant insights

- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/lru_cache/lru_cache_claude-haiku-4-5-20251001_2026-06-06_05-10-02-607/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/lru_cache/lru_cache_claude-haiku-4-5-20251001_2026-06-06_05-10-02-607/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/lru_cache/lru_cache_claude-haiku-4-5-20251001_2026-06-06_05-10-02-607/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/lru_cache/lru_cache_claude-haiku-4-5-20251001_2026-06-06_05-10-02-607/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/lru_cache/lru_cache_claude-haiku-4-5-20251001_2026-06-06_05-10-02-607/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/lru_cache/lru_cache_claude-haiku-4-5-20251001_2026-06-06_05-10-02-607/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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

Exactly one file: `cycle-insight.md` in the output dir.
