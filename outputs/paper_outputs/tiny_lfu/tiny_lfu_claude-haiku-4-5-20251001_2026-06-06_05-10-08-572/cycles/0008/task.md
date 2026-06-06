# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008

## Winners

- v002
- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0008/hypotheses/H03.md

## Prior cycles digest

### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle pursued three orthogonal levers beyond sketch geometry: hash function quality (v002), and two variants of frequency-aware eviction policy (v001, v003). **v001 and v003 advanced as winners, but both came with severe tail-latency costs.** v002, targeting hash function decorrelation, regressed on both hit-rate and latency, revealing that collision reduction is not a limiting factor on this workload. The cohort demonstrates that eviction policy optimization is possible but fragile—hit-rate gains are modest and come at high latency variance costs.

## Patterns

- **Eviction policy improvements trade latency variance for accuracy**: v001 improved hit-rate by ~0.3% through frequency-aware victim selection in a 5-item candidate pool, but introduced +96% tail latency (p99) on put operations. v003 achieved a marginal +0.068% hit-rate gain using median-frequency victim filtering, but paid a catastrophic 364× p99 latency spike. Both variants validate that breaking strict LRU order in favor of frequency signals improves hit-rate, but the scanning/computation overhead during eviction creates latency variance. The baseline's O(1) LRU simplicity purchases latency predictability; frequency-aware selection costs consistency.

- **Hash function quality does not correlate with hit-rate on this workload**: v002's XOR-combined hashing strategy—a standard technique for decorrelating count-min sketch rows—provided no hit-rate improvement and imposed substantial latency cost (36% on get, 27% on put). This suggests the baseline's single `DefaultHasher` with simple seed variation already produces sufficiently independent collisions for this workload, or that collisions are not the primary bottleneck to accuracy. The result contradicts the hypothesis that better hash functions would reduce collision-induced estimation errors.

- **Scaling sensitivity reveals the eviction path is the new bottleneck**: v001's O(1) scanning overhead (5-item pool) is linear in eviction frequency and proportional to cache operations; v003's O(n) median computation and linear victim search saturates at smaller operation volumes. Both exhibit qualitatively different failure modes (v001's gradual degradation vs. v003's catastrophic spike), indicating the eviction code path does not tolerate algorithmic complexity. This marks a shift: prior cycles optimized sketch geometry (collisions); this cycle reveals eviction policy has become the performance constraint.

- **Winners came with liabilities**: Despite advancing, v001 and v003 both exhibit tradeoffs that may not be favorable for latency-critical systems. v001's +0.3% hit-rate gain is offset by +96% tail latency; v003's marginal gain is catastrophically expensive. The "winner" status may reflect hit-rate metrics only, masking latency regressions that would disqualify these variants in production.

## Open questions

- Can v001's frequency-aware selection be tuned to smaller pool sizes (N=2 or 3) to reduce latency variance while retaining most of the hit-rate gain?
- Why did hash function quality not matter? Is the baseline's single seeded DefaultHasher already achieving adequate independence, or does the workload happen to have favorable collision patterns?
- Are the hit-rate improvements from v001 and v003 large enough to justify their latency costs in a latency-critical system? Is there a metric or workload context where these tradeoffs flip?

## Continuation

**Investigate frequency-aware eviction at reduced cost**: v001 proved the concept but at high latency variance. Explore smaller pool sizes (N=2 or 3) on the next cycle to bracket the cost-benefit frontier. **Discard hash function optimization entirely**—v002's failure suggests decorrelation is not a bottleneck. **Reassess whether hit-rate gains are worth latency costs**: if the system prioritizes latency predictability, the strict LRU baseline may be preferable despite lower hit-rate. **Profile the eviction path bottleneck**: determine whether p99 spikes are due to scanning cost, frequency estimation overhead, or something else, then optimize the specific bottleneck rather than changing the policy wholesale. If eviction policy optimization is abandoned, shift focus to orthogonal levers outside the cache internals (e.g., workload-dependent parameter scheduling, admission policy micro-tuning with decay, or cache size variations).


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle tested the operational cost-benefit frontier of frequency-aware eviction through three approaches: minimal pool size (N=2) scanning to reduce latency variance while preserving frequency awareness (v001), revalidation of sketch decay to confirm cycle 0003's benefits (v002), and a combination of decay with N=2 selection to test synergy (v003). **Only v001 and v002 advanced**, with v001 delivering a major latency-variance win (30% p99 reduction on put) at negligible hit-rate cost, while v002 failed to reproduce decay's benefits and confirmed the mechanism is fragile. v003, despite improving hit-rate, incurred catastrophic tail-latency spikes that disqualify it from latency-critical deployment. The cohort reveals that **frequency-aware eviction's true bottleneck is scanning overhead, not the underlying selection logic**, and that the optimal frontier sits at minimal pool sizes, not deeper candidate evaluation.

## Patterns

- **Frequency-aware eviction tuning has shifted from depth to cost**: v001's reduction of the victim candidate pool from N=5 to N=2 eliminated the scanning overhead identified in cycle 0005 (p99 put latency: 570→400 ns, 30% improvement) while hit-rate regressed negligibly (0.7355→0.7354, <0.01%). This validates that checking only the head and next item in the LRU list suffices to identify frequency minima in most cases. The prior cycle's 96% p99 increase from 5-candidate scanning is now resolved by simple pool size constraint. Frequency-aware benefits do not require deep candidate evaluation; the cost is in the mechanism, not the logic.

- **Sketch decay remains a narrow, non-generalizable optimization**: v002's revalidation of decay on the baseline depth-3 geometry failed to reproduce cycle 0003's claimed co-benefits. Hit-rate declined marginally (0.7334→0.7333) and latency increased on both get (+1 ns) and put (+6 ns), contradicting the cycle 0003 narrative of simultaneous improvement. The mechanism adds conditional overhead (op_count checks) and metadata bloat (8-byte field) for negative returns. This confirms cycle 0004's suspicion: decay's cycle 0003 breakthrough was either a statistical artifact of that workload sample or requires highly specific geometry/interval combinations that do not generalize.

- **Composing independent overhead-adding mechanisms amplifies failure modes**: v003 combined decay (500k interval) with N=2 selection, hypothesizing that fresher frequency signals would reduce eviction latency variance. Instead, both mechanisms' overheads stacked: decay's full-sketch traversal every 500k operations created catastrophic tail spikes (+738 ns p99 on get, +1217 ns p99 on put), while frequency-aware selection's benefits were buried under decay's cost. Hit-rate did improve, but the latency variance (put p99 spikes exceeding 1200 ns) violates the latency-critical design principle. Poor composability between mechanisms suggests their benefits are orthogonal or even antagonistic.

- **Hit-rate improvements via eviction policy are expensive in latency-critical systems**: v003 and cycle 0005's frequency-aware policies both improve hit-rate (v003 improves via fresher estimates, cycle 0005 improved via deeper candidate pools), but both incur latency variance costs that may outweigh the gains for systems prioritizing predictability. v001's approach—accept minimal hit-rate loss for major latency-variance recovery—represents a strategic shift in the optimization frontier. The tradeoff has flipped: latency predictability is now the bottleneck, not raw hit-rate.

- **The eviction pool size (N) is a high-leverage tuning dimension**: v001's success with N=2 suggests a sharp dropoff in cost-benefit at the frequency-aware boundary. The LRU head provides one recency candidate; the next item provides one frequency-aware check with minimal overhead. Expanding further (N=3, N=4, N=5) adds quadratic scanning cost but may recover only logarithmic frequency-selection benefit, explaining why v001's N=2 is so effective.

## Open questions

- Is N=2 the true minimum for frequency-aware selection, or is N=1 (pure LRU head, no frequency checks) competitive? v001's result suggests N=2 is optimal, but a direct comparison to N=1 would confirm whether frequency awareness is necessary at all.

- Why does decay fail reproducibly in this cycle but allegedly succeeded in cycle 0003? Is cycle 0003's v003 result a statistical artifact, or are there subtle differences in workload phase characteristics, trace sampling, or prior state that make decay only beneficial under narrow conditions?

- Can the latency variance of frequency-aware selection be reduced through algorithmic changes (e.g., cached frequency estimates, amortized lazy evaluation) rather than pool size reduction?

- Does the LRU-ordered candidate pool (where the head is always the oldest) create an asymmetry that makes N=2 sufficient, whereas random or unordered candidate selection might require larger pools for effective frequency awareness?

## Continuation

**N=2 frequency-aware selection is the practical optimum for latency-respecting eviction**; v001 achieved 30% p99 latency reduction with negligible hit-rate cost, establishing a new baseline. For the next cycle, **test N=1 (pure LRU) and N=3 to bracket the frontier**: N=1 will clarify whether frequency awareness is necessary, and N=3 will show if deeper pools recover material hit-rate gains worth the latency cost. **Discard sketch decay entirely**: v002's failure to reproduce cycle 0003's benefits, combined with measurable overhead on every operation, indicates decay is either workload-specific or a statistical artifact. Unless decay can be implemented with zero conditional cost, it should be deprioritized in favor of provably beneficial levers. **Reject composite mechanisms**: v003 demonstrated that combining independent overhead-adding techniques amplifies worst-case costs without synergy; focus future work on single-axis optimizations or mechanisms with clear composability. **Shift focus to levers outside eviction**: sketch geometry has plateaued (depth-3, width-4 is stable), eviction efficiency has reached diminishing returns (N=2 is efficient), and decay adds no net benefit. The next frontier likely lies in orthogonal dimensions—admission threshold refinement paired with workload-adaptive parameter schedules, hash function cost reduction, or cache-aware instruction profiling to optimize hot code paths.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
