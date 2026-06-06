# Cross-Cycle Insights

## Recurring wins

- **Frequency-aware eviction is load-bearing**: Every attempt to simplify eviction policy (cycles 0005, 0006, 0007) revealed that even minimal frequency-aware victim selection (N=2 candidate pool) actively reduces tail latency and cascading eviction failures compared to pure LRU. Reverting to pure recency (cycle 0007 v001) degraded both hit-rate and latency, confirming the scanning overhead is outweighed by its filtering benefit. The optimal frontier sits at N=2, not deeper.

- **Simple parameter tuning outperforms algorithmic refactoring**: Cycles where constant parameters changed (width-4→3, N=5→2) consistently delivered measurable gains without latency variance. Cycles pursuing code refactors to reduce algorithmic complexity (frequency estimate caching, hash function decorrelation) consistently backfired with microarchitectural penalties (42-139% latency increases). At this operational scale, reducing code surface area beats reducing function calls.

- **Sketch geometry (width/depth) yields high-ROI early, then saturates**: Width reduction 4→3 delivered 17–18% latency improvement with no hit-rate loss (cycle 0007). Earlier width expansion 1→4 (cycle 0001) improved both metrics. But further reductions (cycle 0008) trigger hard boundaries at depth-3, width-3 where 1.2–1.6% hit-rate loss and tail-latency pathology (141% p99 spikes) become unacceptable. Geometry is a high-leverage lever with a fixed frontier.

## Recurring failures

- **Admission threshold micro-tuning is brittle and workload-encoded**: Cycles 0002 (relaxed >=), 0003 (-1 offset), and 0001 (analysis) all attempted to adjust the strict incoming_freq > victim_freq condition. Every relaxation admitted marginal items that displaced higher-value residents, hurting hit-rate. The threshold encodes domain knowledge about workload temperature distribution; marginal adjustments have no smooth gradient. It is a constraint, not a tuning lever.

- **Sketch decay does not generalize and is likely a statistical artifact**: Cycle 0003 v003 appeared to improve both hit-rate and latency simultaneously. Cycles 0004 (all variants with different intervals), 0006 (revalidation on baseline), and later attempts all failed to reproduce the benefit. Decay adds per-operation overhead (conditional branches, metadata checks) and scales poorly with sketch width (cycle 0004 v003 catastrophic tail spike). The cycle 0003 breakthrough was either a lucky workload sample or requires hyperspecific geometry/interval combinations that do not generalize.

- **Composing independent overhead-adding mechanisms amplifies worst-case costs**: Cycle 0006 v003 combined sketch decay (500k interval) with N=2 frequency-aware selection, expecting fresher frequency estimates to improve eviction latency. Instead, both mechanisms' overheads stacked (decay's full-sketch traversal + selection's candidate scanning), creating catastrophic tail spikes (p99 put: +1217 ns). Poor composability suggests the mechanisms' benefits are orthogonal or antagonistic. Future work should focus on single-axis optimizations, not combinations.

- **Hash function decorrelation is not a bottleneck**: Cycle 0005 v002 tested XOR-combined hashing to reduce count-min sketch collisions—a standard technique. It delivered no hit-rate improvement and incurred 27–36% latency cost. The baseline's single seeded DefaultHasher already produces sufficiently independent row hashes for this workload; collisions are not the limiting factor. Decorrelation is a solved problem.

## Open questions

- **Why does sketch decay fail to generalize when cycle 0003 v003 showed apparent improvement?** Was the benefit a statistical artifact of that workload sample, a lucky parameter coincidence (500k interval on depth-3), or a genuine mechanism buried by subsequent noise? Revalidating cycle 0003 v003 in isolation would clarify whether the breakthrough was reproducible.

- **What is the relationship between depth, width, and the workload's Zipfian distribution at the hard frontier (depth-3, width-3)?** Both dimensions independently trigger 1.2–1.6% hit-rate loss and tail-latency pathology when reduced below this point. Is the boundary coupled through collision probability under the workload's characteristic frequency distribution, or independent constraints that happen to align?

- **Can the sketch operation's hot-path cost be reduced through implementation efficiency (vectorization, memory layout, branch prediction) rather than geometric parameter tuning?** If yes, could wider or deeper geometries be viable with lower per-operation overhead, enabling hit-rate improvements without latency penalty.

- **Why does N=2 frequency-aware eviction succeed when larger pools (N=5) fail catastrophically on tail latency?** Is it due to asymmetric LRU-ordered candidate cost (head + next is cheap, subsequent items are expensive), or would unordered/random candidate selection require larger pools for equivalent filtering benefit?

- **Are the observed boundaries (depth-3, width-3, N=2 eviction pool) workload-specific, or would they generalize to other Zipfian traces, cache sizes, or operation frequencies?** All experiments run on a single trace; robustness across broader workload classes is unclear.

## Per-policy notes

- **Admission policy (strict inequality threshold)**: The > condition is load-bearing and workload-optimized. Relaxations to >= or micro-tuned offsets all regressed, admitting marginal items that displaced cache residents unfavorably. No smooth tuning frontier exists; the threshold is a binary constraint. Not a lever for future optimization.

- **Eviction policy (frequency-aware victim selection)**: N=2 frequency-aware scanning is the practical optimum—achieving 30% p99 latency reduction on put operations with negligible hit-rate loss. Pure LRU (N=1) regresses both metrics; larger pools (N=5+) add unacceptable latency variance. The minimal candidate pool (head + next in LRU list) suffices to filter frequency outliers; deeper evaluation does not justify the scanning cost.

- **Sketch geometry (depth and width)**: Depth-3 and width-3 represent the practical frontier for this workload. Reductions below these trigger 1.2–1.6% hit-rate loss and tail-latency pathology. Earlier reductions (4→3 width) delivered wins, indicating the frontier was reached gradually and is now hard. The two dimensions appear coupled through collision probability in Zipfian frequency distributions.

- **Sketch decay (periodic reset)**: Not a viable tuning mechanism for this workload. Decay adds per-operation conditional overhead and scales poorly with sketch width. Cycle 0003's apparent benefits did not reproduce in subsequent validation. If the mechanism is retained, it should be contingent on detecting specific workload conditions (e.g., phase transitions) that justify the cost, rather than applied unconditionally.
