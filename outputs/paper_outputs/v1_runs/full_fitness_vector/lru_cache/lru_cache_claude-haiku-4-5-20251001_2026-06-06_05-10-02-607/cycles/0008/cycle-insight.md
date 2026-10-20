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
