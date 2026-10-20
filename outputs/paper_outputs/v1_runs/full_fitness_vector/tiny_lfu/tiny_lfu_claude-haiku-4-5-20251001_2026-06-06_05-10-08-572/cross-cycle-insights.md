# Cross-Cycle Insights

## Recurring wins

- **Minimal frequency-aware victim selection (N=2)**: Cycle 0006 v001 and validated across cycle 0007–0009. Reduces p99 latency on puts by ~30% with negligible hit-rate cost compared to pure LRU, establishing eviction-policy efficiency frontier. Frequency awareness matters; pool size does not need to exceed 2.

- **Sketch geometry saturation at depth-3, width-3**: Cycles 0007–0008 confirmed this is the practical boundary for this workload. Cycle 0007 v003 (width 4→3) delivered clean 17–18% latency gains; cycle 0008's exhaustive boundary tests show narrower geometry triggers unacceptable 1.2–1.6% hit-rate loss and tail pathology.

- **Asymmetric policies over symmetric thresholds**: Cycle 0009 v003 (distinguish updates from new entries) achieved rare win-win—hit-rate and latency gains simultaneously. Contrasts sharply with cycles 0002–0003's failed micro-tuning of symmetric admission thresholds and cycle 0009's v002 failure with absolute frequency floors.

- **Simple parameter tuning beats complex refactoring**: Cycle 0007 v003's constant change (width reduction) succeeded; v002's algorithmically sound state-management optimization backfired due to microarchitectural penalties. Parameter space exploration remains more reliable than code refactoring at this operational scale.

## Recurring failures

- **Sketch decay is a false positive**: Cycle 0003 v003 claimed breakthrough (hit-rate + latency co-improvement). Cycle 0004 showed catastrophic amplification with wide geometry; cycle 0006 v002 failed to reproduce benefits; v003's composition caused tail-latency spikes >1200ns. Decay adds unconditional overhead for zero net gain; mechanism does not generalize.

- **Sketch geometry expansion at the boundary is counterproductive**: Cycle 0003 v001 (width 4→8 on depth-3) caused 60% latency increase and doubled memory for only 0.6% hit-rate recovery. Cycle 0008 confirmed that pushing narrower (width-2, depth-2) triggers 1.2–1.6% hit-rate loss and severe tail pathology. The frontier is hard-coupled; expansion and contraction both fail.

- **Absolute frequency thresholds and static admission filters**: Cycle 0009 v002's floor (`freq >= 2`) improved average-case but created catastrophic get p99 spike, indicating outlier key access patterns violate the filter and trigger expensive recovery chains. Earlier cycles (0002–0003) showed symmetric threshold micro-tuning is brittle and workload-encoded. Static rules do not capture workload structure.

- **Deep victim candidate pools amplify tail latency**: Cycle 0005 v001 (N=5) caused 96% p99 put increase; v003 (median-frequency filtering) caused 364× p99 spike. Cycle 0006 v001's reduction to N=2 recovered 30% of the lost latency. Scanning overhead scales poorly; minimal pools suffice.

- **Hash function decorrelation does not address the bottleneck**: Cycle 0005 v002's XOR-combined hashing strategy added latency cost with no hit-rate benefit, suggesting collisions are not a limiting factor on this workload or the baseline's seeded DefaultHasher already achieves adequate independence.

## Open questions

- Can cycle 0009 v003's asymmetric admission policy be composed with v001's N=3 eviction pool (or deeper exploration of N) without doubling latency cost or creating pathology? Both target different mechanisms and may be orthogonal.

- Is depth-3, width-3 geometry boundary workload-specific (Zipfian skew, cache size, operation volume), or would it hold across broader trace characteristics? Current evidence is strong for this scenario but generalization is untested.

- Why does asymmetric policy design (semantic awareness of update vs. new entry) succeed when all symmetric threshold tuning fails? Is the workload heavily update-biased, or is asymmetry itself the transferable insight?

- What unexplored admission-policy levers remain? Potential axes: workload-adaptive thresholds (phase-shift detection), cost models incorporating eviction overhead, or frequency-age fusion (recentness and hotness combined).

- Can the sketch query hot path be optimized through microarchitectural means (cache locality, branch prediction, vectorization) to improve per-operation cost without changing geometry, enabling a return to slightly wider sketches if future workloads demand it?

## Per-policy notes

- **Eviction policy (victim selection)**: Progressed from pure LRU to frequency-aware. Optimal frontier at N=2 (minimal scanning). Larger pools (N≥3) show diminishing hit-rate returns (+0.37% at N=3 per cycle 0009) while tail-latency costs escalate (+119ns p99 at N=3). Pure LRU (N=1) regresses on both metrics. Frequency awareness is load-bearing at minimal scope.

- **Sketch geometry (width/depth)**: Stabilized at depth-3, width-3 after cycles 0007–0008's boundary exploration. Early cycles showed width expansion was beneficial; saturation occurred near width-4. Depth reduction from 4→3 (cycle 0002) was profitable; narrower depths trigger collision cascades and tail pathology. Geometric parameters are now frozen; further optimization must come from outside sketch internals.

- **Frequency estimation (sketching)**: Collision-reduction via geometry matters; hash quality does not. Decay mechanisms failed despite theoretical appeal (temporal freshness). The bottleneck is not sketch accuracy itself but how accuracy translates into admission/eviction decisions. Sketch remains the core mechanism; improvements must flow through policy innovation (like cycle 0009's asymmetric admission) rather than sketch tuning.

- **Admission policy**: Transitioned from rigid symmetric rules (>victim_freq) to semantic awareness. Micro-tuning thresholds (cycles 0002–0003, 0009 v002) repeatedly backfired; static floors create outlier pathology. Cycle 0009 v003's asymmetry—always admit updates, stricter gate for new entries—broke this pattern with simultaneous hit-rate and latency gains. Future leverage likely lies in policies that exploit structural workload properties, not global heuristics.
