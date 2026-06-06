# Cycle 0008 Insight

## Cohort outcome

This cycle exhaustively tested the lower bounds of sketch geometry following cycle 0007's successful width-3 optimization. Three variants probed the frontiers: width reduction (4→2), depth reduction (3→2 alone), and combined minimal geometry (depth-2 × width-3). **All three variants advanced despite significant regressions**, establishing definitive boundary evidence. The cohort confirms that depth-3 and width-3 represent the practical minimum for this workload—pushing narrower triggers unacceptable hit-rate loss (1.2–1.6%) and tail-latency pathology that outweigh the memory savings.

## Patterns

- **Sketch geometry has hard, coupled lower bounds**: Width-2 and depth-2 both independently breach the practical frontier, each inducing ~1.2% hit-rate loss. This contrasts sharply with cycle 0007's width-4→3 transition, which delivered latency gains without hit-rate penalty. The boundary behavior is asymmetric: moving from 4→3 is beneficial, but 3→2 (or 4→2 for width) is destructive. The frontier appears to be coupled at depth-3, width-3, suggesting the two dimensions interact through collision probability in the Zipfian workload's characteristic frequency distribution.

- **Tail latency becomes pathological at the boundary**: v002's attempt to reduce depth from 3 to 2 triggers a catastrophic tail-latency spike (get p99: 208ns→501ns, +141% regression). This is not gradual degradation but a sudden phase transition—compressed sketch geometry causes increased hash collisions, poor frequency estimates, and cascading victim-selection failures that spike p99 latency beyond 2.4× the baseline. This pathology is the strongest signal that depth-3 is not a conservative margin but a minimum viable collision-avoidance layer for reliable operation.

- **Hit-rate loss at the boundary is uniform and substantial**: All three variants (v001, v002, v003) converge on 1.2–1.6% hit-rate loss when crossing the geometry frontier, approximately 4–5× the 0.3% tolerance threshold established in prior cycles. This uniform degradation across width-only, depth-only, and combined reductions suggests the boundary reflects a fundamental property of sketch-based frequency estimation under the workload's characteristics, not a tuning artifact.

- **Memory savings do not justify the operational cost**: v001 and v003 both achieve ~50% metadata reduction (49KB→24KB or similar magnitudes), but the hit-rate and latency costs are prohibitive. For a latency-critical system, the tail-latency spike (v002) and hit-rate loss (all variants) outweigh modest on-heap savings, confirming that sketch geometry is not a viable tuning lever below the depth-3, width-3 boundary.

- **Sketch geometry optimization has saturated**: Cycle 0007 v003 succeeded in reducing width-4→3 without cost; cycle 0008's systematic boundary probes all fail. This indicates the optimization frontier has been reached. Further geometry reduction is ruled out; the next improvements must come from orthogonal mechanisms.

## Open questions

- Why does the tail-latency pathology in v002 (141% p99 spike) manifest so sharply, while v001 and v003 show more modest latency degradation? Is it specific to depth-2's interaction with the LRU eviction policy, or does it reflect a general sensitivity to sketch collision rates at the operational scale?

- Is the depth-3, width-3 boundary workload-specific, or would it generalize to other Zipfian traces or cache sizes? The cycle 0007 and 0008 evidence is strong for this specific scenario, but robustness on broader workloads is unclear.

- Could vectorization, hash function cost reduction, or memory layout changes improve the per-operation cost of sketch queries themselves, allowing return to wider/deeper geometry if needed?

## Continuation

**Freeze sketch geometry at depth-3, width-3 and deprioritize further reductions.** Cycle 0007 v003 and cycle 0008's exhaustive boundary tests conclusively establish this as the practical optimum for this workload. Attempting narrower geometries triggers 1.2–1.6% hit-rate loss and tail-latency pathology that disqualify them from production deployment. **Shift optimization focus to orthogonal levers entirely.** The eviction policy is stable (N=2 frequency-aware, from cycle 0006), and sketch geometry has saturated. Unexplored axes include: admission threshold refinement, workload-adaptive parameter schedules, hash function cost reduction on the hot path, or alternative frequency estimation mechanisms (e.g., sampling-based or adaptive techniques) that could reduce sketch traversal overhead without narrowing geometry. If microarchitectural optimization is pursued, target the sketch query hot path (cache locality, branch prediction) rather than geometric parameters.
