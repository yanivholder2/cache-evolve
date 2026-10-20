# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_05-10-08-572/cycles/0010

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_05-10-08-572/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_05-10-08-572/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_05-10-08-572/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_05-10-08-572/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_05-10-08-572/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/tiny_lfu/tiny_lfu_claude-haiku-4-5-20251001_2026-06-06_05-10-08-572/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle tested three refinements to the cycle 0006 baseline (N=2 frequency-aware eviction, depth-3 width-4 sketch): whether pure LRU eviction (N=1) suffices to reach the efficiency frontier (v001), whether caching frequency estimates reduces per-operation redundancy (v002), and whether sketch width can be shrunk from 4 to 3 without hit-rate loss (v003). **v001 and v003 advanced.** v001 definitively proved that even minimal frequency-aware scanning (N=2) is load-bearing, not dispensable overhead; reverting to pure LRU degraded both hit-rate and latency, confirming the eviction policy is near its optimal efficiency. v003 delivered a clean latency win (~17-18% improvement on both get and put) by reducing sketch width, with no hit-rate loss due to power-of-two memory alignment. v002 failed decisively—state-management overhead from caching frequency estimates exceeded the savings of eliminating one redundant sketch query.

## Patterns

- **Eviction policy efficiency is approaching a local optimum at N=2**: v001 tested whether stepping from N=2 (minimal frequency scanning) to N=1 (pure LRU recency) would reduce overhead without hit-rate loss. Instead, both metrics regressed—hit-rate dropped 0.27% and put latency increased 46%. This confirms that frequency-aware victim selection, even with a candidate pool of only 2 items, actively filters out cascading evictions and admission failures that pure recency-based selection permits. The scanning overhead, though real, is outweighed by its filtering benefit. N=2 appears to be a stable frontier: moving toward purer recency harms both latency and accuracy.

- **Microarchitectural effects dominate algorithmic complexity at this operational scale**: v002 attempted a straightforward optimization—cache frequency estimates during victim candidate scanning to avoid redundant sketch queries. The algorithmic change was sound (reduce O(depth) calls), but per-operation latency increased 42-139% across all quantiles. This reveals that compiler optimization, register pressure, branch prediction, or cache effects matter more than call-count reduction for small, fast operations. The original code was either already near-optimal or the refactored state management created microarchitectural penalties that nullified the algorithmic savings. Complex optimizations that add variables or control flow tend to backfire; simple parameter tuning tends to succeed.

- **Sketch geometry (width) remains a high-leverage tuning lever with untapped optimization room**: v003's reduction from width-4 to width-3 delivered ~17-18% latency improvement on both get and put operations with no detectable hit-rate harm (both metrics held constant). The hypothesis expected metadata overhead savings, but the heap footprint remained identical due to power-of-two alignment; the speedup came from a leaner computation path or better cache locality in the smaller sketch. Unlike eviction policy (which is approaching saturation at N=2), sketch geometry still has clear room for efficiency gains.

- **Simple, localized parameter changes outperform complex state-management refactoring**: The cohort's success pattern (v003) vs. failure pattern (v002) reveals a clear asymmetry: changing a constant parameter (width-4 → width-3) delivered measurable gains with no downside, while refactoring code to add caching variables backfired. This suggests that at TinyLFU's scale, reducing surface area (fewer variables, fewer branches) matters more than reducing algorithmic complexity (fewer function calls).

- **Negative results confirm the stability of prior-cycle advances**: v001's failure to improve on N=2 (from cycle 0006) validates that cycle 0006's move to minimal frequency-aware scanning was not a lucky accident—it is a genuine efficiency frontier. Cycle 0006 v001 saved 30% p99 latency on put operations with negligible hit-rate cost; this cycle's attempt to squeeze further by removing frequency awareness altogether confirms that win is structural, not an artifact.

## Open questions

- Is width-3 truly optimal, or can width be reduced further (to 2 or even 1) while maintaining hit-rate? v003's success suggests the current sketch is overprovisioned; the frontier may extend further toward minimal geometry.

- Why does width reduction improve latency when depth reduction (cycle 0002) also delivered gains? Are both dimensions independent, or is there an interaction effect that would make further reductions compound sublinearly?

- Can the sketch computation be made more efficient through hash function changes, vectorization, or memory layout optimizations, allowing a return to wider sketches without latency cost?

- Does N=2 frequency-aware eviction truly represent a hard boundary, or could a probabilistic variant (e.g., checking N=2 candidates with some probability rather than always) recover some of N=1's simplicity while preserving most of N=2's benefit?

## Continuation

**Sketch geometry (width) is still fertile ground for optimization.** v003 proved width-3 improves latency without hit-rate loss; the next cycle should test whether width can shrink further (width-2, width-1) to establish the lower bound. **Confirm that N=2 eviction is optimal and freeze eviction policy**: v001 conclusively demonstrated that frequency-aware scanning at N=2 is load-bearing, not overhead. Accept N=2 as the stable eviction frontier and deprioritize further eviction-policy micro-tuning. **Reject state-management optimizations**: v002's failure shows that adding variables, caching, or refactoring to reduce algorithmic complexity tends to increase microarchitectural cost at this scale. Future work should focus on simple parameter tuning (like v003's constant change) rather than code refactors. **Explore sketch implementation efficiency orthogonal to geometry**: if width-3 is validated as optimal, shift focus to hash function cost, memory layout, or vectorization to improve the per-operation cost of sketch queries themselves, rather than tuning the sketch's width or depth parameters further. If minimal-geometry approaches plateau, the next frontier likely lies in reducing the sketch operation's hot-path cost, not just shrinking its size.


### Cycle 0008

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cycle shifted focus from the now-frozen eviction and sketch-geometry frontiers to admission-policy refinement, the orthogonal lever identified in cycle 0008's continuation. Three variants explored this axis: extending the victim-selection pool from N=2 to N=3 to capture further frequency diversity (v001), adding an absolute frequency floor to filter estimate noise (v002), and implementing an asymmetric admission gate that distinguishes updates to existing items from new entries (v003). **All three advanced**, but with sharply divergent success patterns. v001 achieves modest hit-rate gains (+0.37%) at tolerable latency cost (+119 ns p99 put), v002 fails decisively by improving average-case latency but creating catastrophic tail pathology on gets, and v003 achieves the rare win-win: hit-rate improvement with latency gains across both get and put operations. The cohort reveals that **admission policy design is now the high-leverage frontier, and asymmetric policies that respect workload semantics outperform simple symmetric thresholds**.

## Patterns

- **Asymmetric admission design beats symmetric threshold tuning**: v003's distinction between updates (always admit) and new entries (stricter gate) improved both hit-rate and latency without downside, while v002's absolute frequency floor (admit only if `freq >= 2`) improved average-case performance but introduced severe tail-latency spikes on get operations. The asymmetry in v003 succeeds because it leverages semantic information—existing cache members have proven utility, new items are unproven—whereas v002's symmetric rule treats all items identically regardless of their prior history. Workload-aware policies outperform workload-agnostic heuristics by a wide margin.

- **Simple admission thresholds remain brittle; complexity pays if well-targeted**: Cycle 0008's continuation warned that "admission threshold tuning is brittle and workload-encoded." v002 validates this by adding a second gate that should filter noise but instead creates unpredictable sketch behavior, triggering expensive miss-handling chains when occasional low-frequency keys violate the floor. In contrast, v003's added complexity (a conditional check for key membership) is minimal and targets a specific semantic property (update vs. new entry) rather than a global heuristic. Complexity is only justified if it captures workload structure, not if it adds static filtering rules.

- **Eviction pool size (N) continues to yield hit-rate gains but at increasing latency cost**: v001's extension from N=2 to N=3 improves hit-rate (+0.37%) by identifying colder items deeper in the LRU queue, validating cycle 0006's and cycle 0007's finding that N is exploitable. However, the put p99 regression (+43%) is 4× larger than the hit-rate gain in percentage terms, suggesting the cost-benefit frontier is tightening. The earlier cycle pattern—N=5 caused catastrophic spikes, N=2 was optimal—is now giving way to a more nuanced picture where N=3 is viable but increasingly costly.

- **Tail-latency pathology in v002 is a red flag for threshold-based filters**: The severe get p99 regression in v002 (despite improvements to get p50 and put metrics) indicates that the absolute frequency floor creates outlier cases—occasional low-frequency key lookups that fall below the threshold and trigger cache misses, which in turn trigger expensive sketch operations or eviction chains. This asymmetric pathology (median improves, tail worsens) is characteristic of admission filters that inadvertently create worst-case access patterns. The lesson: symmetric thresholds can hide pathology at the tails.

- **The admission-policy frontier is now more promising than eviction-pool expansion**: Cycle 0008 declared eviction and sketch geometry fully optimized. This cycle confirms that eviction pool expansion (v001) shows diminishing returns, while admission-policy innovation (v003) delivers orthogonal gains. v001 trades latency for hit-rate; v003 improves both. This suggests the system's remaining slack is in the admission layer, not the eviction mechanics.

## Open questions

- Can v003's asymmetric admission gate be combined with v001's N=3 eviction pool to compound the hit-rate gains without doubling the latency cost?

- Is the absolute frequency floor in v002 fundamentally flawed, or would a different threshold (e.g., `freq >= 1.5` or adaptive thresholds based on workload phase) recover hit-rate benefits while avoiding tail pathology?

- What is the upper bound on N for victim selection? v001 tests N=3; does N=4 remain tolerable, or does it approach N=5's catastrophic regime?

- Why does v003's semantic distinction (update vs. new entry) work so cleanly when simple threshold rules consistently fail? Is the workload heavily biased toward update-heavy access patterns, or is asymmetry itself the key insight?

## Continuation

**Adopt v003's asymmetric admission policy as the new baseline.** It achieves hit-rate and latency gains simultaneously, a rare win that justifies immediate integration. The key insight—existing cache residents deserve preferential treatment over unproven newcomers—is both intuitive and effective. **Test the composition of v003's admission policy with v001's N=3 eviction pool** in the next cycle to determine if the gains compound or interact negatively. v003 and v001 target different mechanisms (admission vs. eviction); their orthogonality should preserve both benefits. **Deprioritize absolute-threshold admission filters** unless they can be made workload-adaptive. v002's failure to improve hit-rate and tail-latency pathology indicate that static frequency floors are not an effective lever; the workload does not benefit from filtering low-frequency admissions directly. **Shift focus to workload-semantic policies**: the success of v003 suggests future improvements lie in policies that explicitly model entry type, access history, or other semantic properties rather than global heuristics. If further admission policy work is pursued, prioritize policies that distinguish different classes of operations or exploit structural properties of the workload, not symmetric threshold rules.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
