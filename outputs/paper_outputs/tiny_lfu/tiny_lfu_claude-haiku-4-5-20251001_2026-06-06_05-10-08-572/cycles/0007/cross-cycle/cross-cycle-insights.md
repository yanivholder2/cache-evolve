# Cross-Cycle Insights

## Recurring wins

- **Minimal sketch geometry (depth 3–4, width 3–4) is a stable, efficient frontier**: Cycles 0002 (depth 4→3: 25% latency gain), 0003 (width-4 paired with depth-3), and 0007 (width 4→3: 17–18% latency gain) all show that modest, targeted reductions in sketch size yield clean latency improvements with negligible or zero hit-rate loss. Expansion in either direction (0001, 0003, 0004) consistently backfires, indicating the baseline sketch is near-optimal for this workload.

- **Frequency-aware victim selection at minimal pool size (N=2) balances accuracy and latency**: Cycles 0006–0007 converged on N=2 as the optimal eviction frontier. v001 in 0006 delivered 30% p99 latency reduction on put operations with <0.01% hit-rate loss; v001 in 0007 confirmed that stepping to N=1 (pure LRU) regressed both metrics, proving frequency-aware filtering remains load-bearing even at minimal cost. The candidate pool size is a high-leverage tuning dimension.

- **Simple parameter tuning consistently outperforms code refactoring**: Changing a constant (e.g., width-4 to width-3 in cycle 0007 v003) yields measurable gains, while adding state-management variables (cycle 0007 v002's frequency caching) increases latency by 42–139% due to microarchitectural effects. At this operational scale, algorithmic complexity reduction is dominated by branch prediction, register pressure, and cache locality.

- **Frequency estimation quality is the foundational lever**: Cycle 0001 established that accuracy in frequency counts (via width/collision reduction) drives downstream performance more than per-operation latency. All subsequent cycles built on this insight; sketch geometry and eviction policies refine rather than replace frequency-based filtering.

## Recurring failures

- **Sketch decay is a brittle, non-generalizable optimization**: Cycle 0003 appeared to achieve a breakthrough (both hit-rate and latency improved simultaneously), but cycles 0004 and 0006 failed to reproduce the benefit. Aggressive decay intervals (250k) add measurable overhead; conservative intervals (1M) are ineffective; and pairing decay with wider geometries (0004 v003, 0006 v003) creates catastrophic tail spikes. The cycle 0003 victory was either a statistical artifact or confined to a narrow parameter space. Decay adds conditional overhead on every operation; its cost is structural, not tunable.

- **Admission threshold relaxation consistently fails despite improved frequency estimation**: Cycles 0001–0003 show that relaxing the threshold from `incoming_freq > victim_freq` to `>=` or `>= victim_freq - 1` admits marginal items that do not justify displacing cache residents, degrading hit-rate. The strict inequality encodes workload-specific temperature distribution knowledge that marginal threshold adjustments cannot overcome. The threshold is a brittle, workload-dependent local optimum with no exploitable gradient.

- **Geometry expansion adds cost faster than it recovers accuracy**: Width expansion on stable depth-3 baseline (0003 v001) increased latency by 60% and memory by 2× for ~0.6% hit-rate gain. Depth expansion (0002 v003) added 11% latency and 50% memory for +0.33% hit-rate. Collision reduction via deeper or wider sketches hits diminishing returns quickly; the baseline geometry is already near the cost-benefit frontier.

- **Hash function decorrelation is not a bottleneck**: Cycle 0005 v002 tested XOR-combined hashing to reduce cross-row collisions, expecting hit-rate improvement. Instead, it imposed 27–36% latency cost with no hit-rate gain, indicating the baseline's single seeded `DefaultHasher` with simple seed variation already produces sufficient independence for this workload, or collisions are not the primary bottleneck to accuracy.

- **Complex mechanisms do not compose**: Cycle 0006 v003 combined decay (500k interval) with N=2 frequency-aware selection, hypothesizing that fresher frequency signals would reduce eviction latency variance. Instead, both mechanisms' overheads stacked catastrophically (+1217 ns p99 spike on put). Independent overhead-adding optimizations tend to amplify costs rather than yield synergy.

## Open questions

- **Can sketch geometry shrink further than depth-3, width-3?** Cycles 0002 and 0007 validated modest reductions; the frontier may extend toward minimal sketches (width-2 or even width-1). Direct testing would establish whether the baseline is truly at the efficiency boundary or if further compression is possible.

- **Is the admission threshold fundamentally brittle, or do workload phases unmask exploitable structure?** Cycles 0001–0003 showed micro-tuning failed on a single workload baseline. If decay's (failed) temporal freshness argument held merit, perhaps adaptive thresholds tied to workload phase transitions could succeed where static offsets failed. This would require workload-adaptive parameter schedules, not single-axis tuning.

- **Can sketch operations be optimized orthogonal to geometry (width/depth)?** Cycle 0007 v002's failure to improve via algorithmic simplification suggests the bottleneck is microarchitectural, not algorithmic. Hash function cost, memory layout, vectorization, or cache-aware instruction profiling might improve per-operation latency without shrinking the sketch further.

- **What is the true lower bound on eviction pool size?** Cycle 0006 v001 succeeded at N=2; cycle 0007 v001 showed N=1 regresses both metrics. Is N=2 a hard boundary where LRU ordering provides recency and the next item provides one frequency check, or could probabilistic candidate sampling recover some N=1 simplicity while preserving N=2's filtering benefit?

- **Why does frequency-aware eviction improve hit-rate despite modest gains?** Cycles 0005–0007 show that checking 2 LRU candidates for frequency minima recovers 0.27% hit-rate (cycle 0007 v001). The mechanism filters cascading evictions and misallocations; understanding why such minimal candidate pools suffice would inform whether deeper scanning could recover material gains under different workload characteristics (burstiness, phase transitions, lower Zipfian skew).

## Per-policy notes

- **LRU with N=2 frequency-aware victim selection (depth-3, width-3 sketch)** is the converged optimum across cycles 0006–0007. This configuration balances accuracy (filters cascading evictions), latency (minimal scanning overhead), and memory (minimal geometry). Reverting to pure LRU (N=1) loses frequency signal; expanding to N=5 incurs unacceptable p99 spikes. Accept this as the stable eviction frontier.

- **Pure LRU baseline is a low-latency but low-accuracy baseline**: Cycles 0006–0007 show that removing even minimal frequency-aware filtering (N=2 → N=1) regresses both hit-rate and latency, contradicting the intuition that simpler policies are faster. This suggests frequency-aware filtering prevents cascading evictions that harm tail latency more than scanning overhead costs.

- **Frequency-aware eviction's tail-latency variance is load-bearing at larger pool sizes**: Cycles 0005–0006 show that N=5 candidate scanning incurs +96% p99 spikes on put operations, while N=2 reduces variance by 30% relative to the baseline. The scanning cost is not linear; it depends on candidate order (LRU head benefits from spatial locality) and eviction frequency (more evictions = more scans under contention).

- **Decay mechanisms should be abandoned as a standalone tuning lever**: Cycles 0003–0007 show decay either fails reproducibly (0004, 0006), adds pure overhead (0007 v002), or creates catastrophic tail spikes when composed with other mechanisms (0006 v003). Unless decay can be implemented with zero conditional cost (e.g., FIFO counter overflow as a side effect), deprioritize it entirely in favor of provably beneficial levers like geometry tuning or eviction pool sizing.
