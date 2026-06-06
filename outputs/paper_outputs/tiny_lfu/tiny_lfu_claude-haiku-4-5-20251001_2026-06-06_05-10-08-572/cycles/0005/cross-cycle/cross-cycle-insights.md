# Cross-Cycle Insights

## Recurring wins

- **Width-based collision reduction is high-ROI**: Cycle 0001's sketch width expansion (SKETCH_WIDTH_FACTOR=8) simultaneously improved hit-rate and per-operation latency, a pattern validated by the depth-3 baseline's stability through subsequent cycles. Geometry improvements that reduce hash-row collision density pay off consistently.

- **Moderate depth reduction (depth 3) enables clean trades**: Cycle 0002's depth reduction from 4 to 3 delivered ~25% latency gain for <0.3% hit-rate loss. This trades one metric for the other predictably, establishing depth-3 as a stable operating point that survives further optimization attempts.

- **Admission threshold remains workload-optimal**: The strict `incoming_freq > victim_freq` condition—never relaxed successfully—acts as a load-bearing constraint. Every attempt to relax it (cycle 0002 v001, cycle 0003 v002) failed consistently, suggesting the threshold encodes domain knowledge about workload temperature distribution that marginal improvements to frequency estimation cannot override.

## Recurring failures

- **Admission policy relaxation collapses despite improved frequency estimates**: Cycles 0002 and 0003 both tested relaxing the admission barrier (>= instead of >), expecting that better frequency signals would justify admitting more items. Both failed by admitting borderline items that displaced higher-value cache residents. The strict threshold appears orthogonal to sketch quality and cannot be traded away.

- **Depth expansion and excessive width show diminishing returns**: Cycle 0002's depth-6 expansion (+0.33% hit-rate, +11% latency, +50% memory) and cycle 0003's width-8 on depth-3 baseline (+0.6% hit-rate, +60% latency, 2× memory) both demonstrate that collision reduction saturates. Beyond a certain point, sketch geometry cannot buy further accuracy improvements.

- **Sketch decay is fragile and workload-specific**: Cycle 0003's decay appeared as a breakthrough (both hit-rate and latency improved on depth-3 baseline with 500k interval), but cycle 0004 revealed brittleness: all three decay-frequency variants (250k, 1M, and 500k on wider geometry) regressed or showed mixed results. The decay mechanism's overhead is proportional to sketch size, making it antagonistic with wide geometries. The cycle-0003 victory appears to be a narrow optimum, not a generalizable pattern.

- **Hash function improvements do not correlate with hit-rate**: Cycle 0005's v002 tested decorrelated hashing (XOR-combined multi-seed approach) and regressed on both hit-rate and latency. The baseline's simple seed-varied hashing appears sufficient, suggesting collisions are not the primary bottleneck to frequency estimation accuracy on this workload.

- **Frequency-aware eviction trades unacceptable latency variance for modest hit-rate gains**: Cycle 0005's v001 and v003 improved hit-rate (+0.3% and +0.068%, respectively) but incurred catastrophic tail-latency costs (+96% and 364× p99 spikes). The scanning and frequency-computation overhead during eviction becomes a bottleneck, making strict O(1) LRU preferable if latency consistency is valued.

## Open questions

- **Can decay on depth-3 width-4 with 500k interval be reproduced?** Cycle 0004 suggested cycle 0003's victory was a statistical artifact or narrow optimum. Revalidation on the stable baseline would clarify whether decay is a genuine lever or a false positive.

- **Is there an optimal frequency-aware eviction pool size that balances hit-rate gains and latency?** Cycle 0005's v001 scanned 5 items and achieved +0.3% hit-rate at +96% p99 cost. Testing smaller pools (N=2 or 3) could bracket the cost-benefit frontier and identify whether frequency-aware selection is viable at latency-critical operating points.

- **Does sketch geometry saturation require shifting optimization focus?** Five cycles of geometry tuning (width, depth, decay) have reached diminishing returns. Are the next ROI improvements outside the sketch parameters—victim selection, hash functions, adaptive policies, or workload-dependent scheduling?

- **What workload characteristics make decay beneficial vs. detrimental?** Decay improved both metrics on a narrow baseline but failed elsewhere. Understanding whether success correlates with phase transition frequency, operation volume, or workload skew could guide redeployment.

## Per-policy notes

- **Admission policy (strict >)**: The strict inequality condition is optimal and orthogonal to sketch quality. Relaxation attempts (two cycles) failed consistently. Micro-tuning this threshold should be abandoned; the baseline appears workload-optimized. Consider alternative axes (e.g., decay coupled with strict admission, adaptive thresholds based on phase detection) rather than direct threshold relaxation.

- **Eviction policy (LRU baseline)**: Simple O(1) LRU dominates frequency-aware variants on latency-critical metrics. Hit-rate improvements from frequency-aware selection are modest (<0.3%) and come at unacceptable tail-latency cost (>90% p99 spikes). Revisit only if latency sensitivity changes or if a sub-linear-cost variant is discovered.

- **Sketch geometry**: Optimal baseline emerging as depth-3, width-4. Width expansion is high-ROI up to width-4 (cycle 0001 demonstrated this); depth expansion is low-ROI (cycle 0002 v003). Further expansion in either dimension triggers diminishing returns or cost barriers. Decay paired with this baseline is unresolved (cycle 0004 raised reproducibility questions).

- **Frequency estimation (count-min sketch)**: The sketch's accuracy appears sufficient with baseline geometry (width-4, depth-3). Decay's mechanism (temporal freshness) is orthogonal to static collision reduction; if decay is deployed, it should be paired with stable geometry, not used as a tuning lever for geometry itself. Hash function improvements do not help; stick with simple seed-varied approach.
