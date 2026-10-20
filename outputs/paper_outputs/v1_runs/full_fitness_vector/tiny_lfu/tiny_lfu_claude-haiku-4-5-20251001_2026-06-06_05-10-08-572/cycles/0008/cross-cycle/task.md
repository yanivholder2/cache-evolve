# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three independent levers were explored: sketch width (v001), sketch depth (v002), and admission barrier relaxation (v003). All advanced to winners, validating distinct dimensions of cache tuning. The cohort demonstrated that **frequency estimation quality is the primary lever**—investing memory to reduce collisions (v001) paid off, while aggressive cuts to estimation accuracy (v002) cascaded into operational degradation despite median latency gains.

## Patterns

- **Frequency accuracy drives overall performance more than per-operation latency**: v002 reduced sketch depth, trimming per-operation work and improving get p50, but hit-rate and tail latencies deteriorated because poor frequency estimates drove worse admission decisions and more evictions. This suggests the cache bottleneck is decision quality, not hash computation.
- **Memory spent on estimation accuracy yields multiplicative returns**: v001 doubled memory usage but improved both hit-rate and per-op latency simultaneously, indicating that hash collision reduction on Zipfian workloads justifies the metadata cost.
- **Admission policy is an independent tuning axis**: v003 showed that operational throughput (~17% latency improvement) can be purchased by relaxing the admission barrier, independent of sketch structure. This trades hit-rate for admission efficiency but operates orthogonally to frequency estimation.
- **Tail latency is a better fitness signal than median**: v002's degradation in p99 and put operations—despite p50 improvement—revealed that poor frequency estimates create adverse cache states (more evictions, more puts) that hurt the tail, not just the median.

## Open questions

- Does v001's width increase pair well with v003's relaxed admission? The two moved in opposite directions on hit-rate; combining them might find a balanced sweet spot.
- What is the lower bound on sketch depth without catastrophic accuracy loss? v002 went to 2; does 3 or intermediate values preserve hit-rate while yielding latency gains?
- On denser workloads (lower Zipfian skew), do frequency-based decisions remain dominant, or does operational latency become the limiting factor?

## Continuation

Prioritize **frequency estimation investment** over raw per-operation cost reduction. v001's approach proved that memory spent on collision avoidance is well-spent on Zipfian traces. For cycle 0002, explore moderate depth reduction (depth 3–4) paired with v001's wider width, and test whether v003's relaxed admission compensates for hit-rate erosion in the ensemble. Investigate whether sketch dimensions should adapt to workload characteristics (high vs. low Zipfian skew).


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle pursued two orthogonal directions: admission policy refinement (v001) and sketch geometry optimization (v002 and v003). v001 and v002 advanced; v003 dropped. **The admission threshold proved inflexible**: pairing cycle-0001's superior frequency estimation with relaxed admission (>=) backfired, admitting borderline items that degraded hit-rate despite the wider sketch's improved accuracy. Meanwhile, **moderate depth reduction is viable**: v002 achieved a favorable latency-accuracy trade, cutting per-operation work by 25% for only ~0.3% hit-rate loss.

## Patterns

- **Admission barrier is load-bearing; frequency accuracy alone does not overcome it**: v001 combined v001's wider sketch (SKETCH_WIDTH_FACTOR=8, which improved both hit-rate and latency in cycle 0001) with the relaxed admission policy, expecting synergy. Instead, hit-rate declined because the lowered threshold admitted items not hot enough to justify displacing cache residents. This reveals that the strict threshold (incoming_freq > victim_freq) encodes domain knowledge about workload temperature distribution that marginal frequency improvements cannot override.

- **Depth reduction has a sweet spot around 3 layers**: v002 reduced depth from 4 to 3 and delivered a clean trade—25% per-operation latency gain (get: 132→90 ns) for <0.3% hit-rate loss. This vindicates cycle 0001's open question about the lower bound on sketch depth and challenges v003's expansion approach.

- **Depth expansion yields diminishing returns**: v003's increase to 6 layers delivered only +0.33% hit-rate improvement but paid 11% latency cost and 50% more memory. This suggests collision reduction is no longer the bottleneck at depth 4, and that frequency estimation accuracy saturates before hitting depth constraints.

- **Width and depth are not interchangeable levers**: Cycle 0001 showed width expansion is high-ROI; cycle 0002 shows depth expansion is low-ROI. The cohort indicates the cache bottleneck is not hash-row collisions (depth) but rather width-based collision density within rows.

## Open questions

- Can we recover v002's hit-rate loss by pairing depth 3 with increased width? A narrower sketch with more independent hashing might compound benefits from both directions.
- Does the admission threshold have a gradient between > and >=? Is there an intermediate condition (e.g., incoming_freq >= victim_freq - 1) that admits slightly more while preserving hit-rate?
- Why did relaxing admission with better frequency estimates still hurt, when cycle 0001's v001 standalone improved hit-rate? Did the wider sketch lose some advantage when paired with a permissive threshold, or is admission policy truly dominant?

## Continuation

**Pursue width-based optimization on a depth-3 baseline**: v002 proved depth 3 is stable for latency-accuracy balance. Test whether SKETCH_WIDTH_FACTOR expansion (e.g., 6–8) on depth 3 recovers hit-rate while preserving latency gains. **Abandon pure depth exploration**; v003's expansion shows diminishing returns and v002's moderate reduction is the viable direction. **Revisit admission policy as an independent lever**: the strict > threshold appears overfit to this workload; micro-tune it in isolation (without width changes) to understand its range. If possible, isolate whether the failure of v001 was due to frequency estimation or admission permissiveness.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle tested three orthogonal hypotheses: width-4-to-8 expansion on depth-3 (v001), micro-tuning the admission threshold with a -1 offset (v002), and periodic frequency sketch decay (v003). **Only sketch decay advanced as a winner**—the first variant in three cycles to improve both hit-rate and per-operation latency simultaneously. Width expansion proved counterproductive, and admission threshold tuning remains brittle. The cohort reveals that sketch geometry has reached saturation while temporal freshness of frequency estimates opens a new, unexplored dimension.

## Patterns

- **Width expansion hits diminishing returns and cost barriers on depth-3 baseline**: v001 doubled width (4→8) to recover hit-rate lost in cycle 0002's depth reduction, successfully raising hit-rate by ~0.6%. However, per-operation latency surged 60% (90→144 ns) and memory doubled, negating the latency gain that depth-3 achieved. This contradicts cycle 0001's finding that width expansion is high-ROI, suggesting the cost-benefit ratio is cache-size and workload specific. At this cache capacity, width 4 is near-optimal.

- **Admission threshold remains a local optimum with no smooth gradient**: v002 tested a -1 offset (>= victim_freq - 1) as a middle ground between strict (>) and permissive (>=). It failed predictably: the relaxation admitted marginal items that displaced higher-value cache residents, hurting hit-rate (opposite of the hypothesis). This mirrors cycle 0002's failure with >=, indicating the strict inequality encodes workload-specific temperature distribution knowledge that marginal relaxations cannot overcome. The threshold is brittle and workload-dependent.

- **Sketch decay is a breakthrough orthogonal axis**: v003's periodic decay (every ~500k operations) improved both hit-rate and latency simultaneously, a first across three cycles. Hit-rate gains are modest but consistent (responsive to workload phase changes), while latency improvements are substantial—both get (132→112 ns) and put (166→142 ns) operations accelerated. The mechanism appears to be two-fold: (1) temporal freshness for adaptive admission, and (2) reduced computational overhead as unbounded count growth is periodically reset. This suggests decay addresses a bottleneck orthogonal to sketch geometry.

- **Latency-accuracy tradeoffs are not absolute**: Most prior optimizations traded one metric for another. v003 broke this pattern by improving both, hinting that sketch staleness is a distinct constraint from collision-density (geometry) or selectivity (admission).

## Open questions

- Does sketch decay synergize with width-4, depth-3 geometry (the likely stable baseline)? v003 was tested against the baseline; combining it with known-good geometry might compound gains.
- What is the optimal decay frequency for this workload? v003 used ~500k operations; is this interval universal or workload-dependent? Longer intervals might reduce overhead further; shorter intervals might improve phase responsiveness.
- Does decay scale to different cache sizes and Zipfian skews? All variants have been tuned on one trace; decay's responsiveness benefit may be more valuable on bursty or phase-shifting workloads.
- Why did admission micro-tuning fail when sketch geometry tuning succeeded? Is the threshold truly load-bearing, or are the workload's phase transitions (which decay now handles) the hidden reason prior threshold tuning failed?

## Continuation

**Depth-3, width-4 baseline is near-optimal for sketch geometry**; v001's width expansion proved too costly, and v002's depth reduction already extracted the latency gain. Stop pursuing geometry changes. **Decay has opened a new frontier**: v003's co-benefits (hit-rate + latency improvement, not just tradeoffs) suggest temporal freshness is a first-class tuning lever. For cycle 0004, test decay decay-frequency tuning (e.g., 250k and 1M operations to bracket v003's 500k), and consider pairing optimal decay with a stable geometry baseline to measure compounding effects. **Abandon admission threshold micro-tuning**—the strict `>` condition is workload-optimized and has no smooth gradient. Shift focus entirely to decay mechanics and interactions with sketch geometry.


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


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
