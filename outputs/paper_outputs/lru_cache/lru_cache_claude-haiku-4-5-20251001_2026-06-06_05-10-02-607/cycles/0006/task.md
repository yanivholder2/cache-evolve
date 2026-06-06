# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-06-06_05-10-02-607/cycles/0006

## Winners

- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-06-06_05-10-02-607/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-06-06_05-10-02-607/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-06-06_05-10-02-607/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-06-06_05-10-02-607/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-06-06_05-10-02-607/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-06-06_05-10-02-607/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
