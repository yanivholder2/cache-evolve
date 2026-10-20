# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0007

## Winners

- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle tested whether sketch decay—identified as a breakthrough lever in cycle 0003—could be optimized through decay-frequency tuning and synergy with high-frequency geometry. Three decay variants were deployed: aggressive 250k-operation intervals on the stable depth-3 baseline (v001), conservative 1M-operation intervals on the same baseline (v002), and 500k decay paired with the wider width-8 geometry from cycle 0001 (v003). **All three regressed or showed mixed results.** The aggressive interval (v001) traded the depth-3 baseline's latency advantage for marginal hit-rate gains; the conservative interval (v002) added pure overhead with no hit-rate benefit; and decay paired with wide geometry (v003) created catastrophic tail-latency spikes without hit-rate recovery. The cohort reveals that **sketch decay's benefits are fragile and tightly coupled to specific geometry choices**, not a universally applicable tuning lever.

## Patterns

- **Decay overhead is proportional to sketch size and operation frequency**: v001 added modest latency (+3–7 ns per operation) on the smaller width-4 sketch; v003's decay on width-8 geometry created massive tail-latency degradation (p99 get: 154→243 ns). The O(depth×width) bitwise reset operation is expensive at scale, and checking on every operation exacerbates the cost. This suggests decay is fundamentally misaligned with high-frequency sketches.

- **Decay interval thresholds operate in a tight range with no smooth gradient**: v002's conservative 1M interval rarely triggers on this workload, paying conditional overhead for zero hit-rate benefit—a pure fixed cost. v001's aggressive 250k interval triggers frequently enough to degrade latency visibly. Neither bracket the original cycle-0003 sweet spot of 500k, suggesting that point was either optimal by accident or was the result of orthogonal factors (e.g., workload phase transitions) not present on all traces.

- **Decay does not fix collision or admission problems in wide sketches**: v003 expected that fresher frequency signals would help the width-8 geometry avoid noisy estimates during phase transitions. Instead, the decay mechanism's overhead dominated, and hit-rate degraded (0.7409→0.7405), indicating stale counts were not the limiting factor—collision density and width-based accuracy were. Decay appears orthogonal to, or even antagonistic with, geometry-based tuning.

- **Cycle 0003's decay victory may have been workload-specific or a lucky local optimum**: v003 improved both hit-rate and latency on a narrow baseline (depth-3, width-4, 500k interval). Cycle 0004 shows that result does not generalize: shorter intervals add overhead without benefits, longer intervals are ineffective, and wider geometries amplify the cost. The prior cycle's "breakthrough" appears to have been fragile, confined to a specific parameter space rather than a robust, composable lever.

## Open questions

- Was cycle 0003's v003 (500k decay on depth-3 baseline) a genuine improvement or a statistical artifact? All attempts to re-tune or compose it failed. Should the baseline be re-validated to confirm the decay benefit was real?

- Is there a decay interval sweet spot between 250k and 1M that was never tested? v001 and v002 bracket the space but don't hit the 500k midpoint on the depth-3 baseline. If 500k is truly optimal, what property of the workload (operation volume, phase transition frequency) makes that threshold special?

- Why does decay's overhead scale so poorly with sketch width? v003's tail-latency spike suggests the bitwise right-shift on 32 counters (8×4 = width×depth) is expensive under contention. Could decay be implemented more efficiently (e.g., lazy resets, probabilistic sampling) to reduce per-operation cost?

## Continuation

**Decay as a tuning lever has reached diminishing returns and is brittle across parameter variations.** The cycle 0003 victory was likely a narrow optimum, not a robust pattern. **Stop pursuing decay frequency micro-tuning.** Instead, isolate decay's true benefit: re-validate cycle 0003's v003 on the depth-3 width-4 baseline with 500k interval to establish whether the gain was reproducible and how large it actually was. If genuine, **accept depth-3 width-4 with optional 500k decay as the stable geometry baseline** and shift focus to orthogonal levers: eviction policy variations, hash function choices, adaptive admission thresholds, or workload-dependent parameter schedules. If decay was a false positive, **discard it entirely and focus on unexplored dimensions** (e.g., victim selection, frequency aging strategies not based on periodic resets, or cache-oblivious tuning). Geometry optimization has plateaued; the next frontier is likely outside the sketch parameters.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
