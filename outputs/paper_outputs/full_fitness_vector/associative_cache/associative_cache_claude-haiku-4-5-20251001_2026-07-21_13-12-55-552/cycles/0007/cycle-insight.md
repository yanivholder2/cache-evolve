# Cycle 0007 Insight

## Cohort outcome

This cohort tested three orthogonal directions to overcome bottlenecks identified in prior cycles: partitioning capacity constraints in adaptive designs, micro-optimizing rare eviction paths, and rebalancing probe distances to reduce latency variance. All three variants regressed or produced pyrrhic gains (e.g., hit-rate improvement at 2–3× latency cost). No variant achieved net fitness improvement.

## Patterns

- **Capacity is a structural constraint, not an algorithmic problem**: v001 revealed that cycle 0006 v003's hit-rate gain depended critically on the large hot-partition buffer, not on the frequency-aware algorithm itself. Reducing buffer size from 60% to 12.5% erased the hit-rate gain and tripled latency—proof that adaptive partitioning overhead becomes untenable when partition size shrinks. Algorithmic elegance cannot substitute for sufficient working-set capacity.

- **Micro-optimization on infrequent paths has unintended compiler consequences**: v002 successfully reduced put latency by replacing functional patterns with explicit loops in the eviction code. However, structural changes to the eviction module disrupted the compiler's optimization of the hit path, causing get latency to regress. This pattern (orthogonal improvement causes cross-path regression) repeats from cycle 0005 and confirms that code organization affects compiler inlining and binary layout in ways difficult to predict or control.

- **Hit-rate gains are not valuable if latency doubles**: v003 achieved the strongest hit-rate improvement (+8.6%) by rebalancing probe distances via Robin Hood insertion, but at the cost of near-doubling both get and put latencies. The algorithmic benefit was Pareto-dominated by the latency cost, revealing a false improvement—higher hit-rate that no workload can afford to use.

- **Every structural change exposes the cost/benefit asymmetry**: All three variants attempted to improve a secondary dimension (capacity overhead, eviction latency, probe variance) while preserving the primary dimensions (hit-rate, hit-path latency). Each revealed that the overhead of the structural change exceeded the benefit it delivered. Cycles 0004–0007 collectively show that adding bookkeeping, tracking fields, or rebalancing logic costs more in common cases than it saves in rare cases.

## Open questions

- Is there any vectorization or batching opportunity that could amortize the overhead of Robin Hood insertion, making v003's hit-rate gain achievable at acceptable latency?
- Why does cycle 0006 v003 remain competitive (hit-rate 0.6988) despite its acknowledged overhead, while cycle 0007 v003 (Robin Hood) regresses? Does hit-rate ceiling masking different latency/capacity trade-offs?
- Have all reasonable orthogonal directions (capacity, path isolation, probe variance) been exhausted? What structural or algorithmic degrees of freedom remain unexplored?
- Should the next cycle return to validating cycle 0004 v003's fitness across different workloads and cache sizes (Path A), or is the cost of micro-tuning so high that only radical departures (learned strategies, alternative data structures) can progress?

## Continuation

Cycle 0007 confirms that orthogonal micro-optimizations to overhead, variance, or capacity constraints all fail: either they regress because of compiler side effects, or they trade hit-rate for latency in ways that leave fitness unchanged or worse. The cohort reinforces cycle 0006's finding that single-threaded, Zipfian, fixed-size cache optimization is resource-constrained by modern CPU memory hierarchies and compiler behavior.

Next cycle should: **(1) Validate whether cycle 0004 v003 or cycle 0006 v003 is truly a fitness leader** across multiple workload distributions (not just Zipfian) and cache sizes to determine if Path A (robustness) is justified; **(2) if validation confirms a prior leader, shift focus away from optimization toward robustness and failure-mode identification**—when does the leader break?; or **(3) if no clear leader emerges, pivot decisively to Path B (orthogonal approaches)—learned insertion heuristics, cuckoo hashing, cache-oblivious designs, or multi-core efficiency**. Do not continue refinement of capacity, probe variance, or eviction-path micro-tuning; all three are now empirically closed.
