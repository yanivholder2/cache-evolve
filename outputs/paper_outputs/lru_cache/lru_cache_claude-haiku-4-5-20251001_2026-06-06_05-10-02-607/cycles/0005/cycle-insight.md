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
