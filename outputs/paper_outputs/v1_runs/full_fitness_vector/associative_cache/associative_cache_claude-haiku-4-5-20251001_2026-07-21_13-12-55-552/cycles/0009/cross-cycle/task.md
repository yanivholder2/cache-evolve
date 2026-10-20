# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

This cohort tested three distinct improvements to ClassicLRU: a frequency-aware eviction policy (LFU), a refined bucket structure, and an asymptotically faster removal mechanism (Swap). All three variants degraded latency performance relative to baseline, and none improved hit-rate on the Zipfian workload. The cohort demonstrates that theoretical advantages—frequency awareness, finer-grained bucket isolation, O(1) removal—do not survive contact with implementation overhead and memory access patterns in practice.

## Patterns

- **Latency regressed uniformly**: LFU, bucket repartitioning, and Swap each introduced measurable per-operation costs (ranging from 45% to 3× slowdowns), indicating that any added bookkeeping or structural change incurs penalties that the baseline already avoids.
- **Hit-rate was flat or worsened**: Two variants maintained 0.6611 while one degraded to 0.6568, suggesting that ClassicLRU's simple recency-based eviction already aligns well with Zipfian access patterns and that the workload does not reward frequency tracking or fine-grained bucket segregation.
- **Theory vs. practice gap widest for Swap**: The policy's asymptotic O(1) removal advantage (vs. ClassicLRU's O(n)) inverted at the benchmark scale, implying that constant factors—cache coherency, memory latency, swap chain overhead—dominate actual performance for this geometry.
- **Baseline stability**: The consistent hit-rate of 0.6611 across variants suggests the workload's access distribution is captured by recency alone, making the ClassicLRU baseline robust for this scenario.

## Open questions

- Would batched or vectorized updates to frequency counters (or swap chains) recover performance while retaining the semantic benefits of LFU or Swap?
- Does ClassicLRU's dominance hold across other access distributions, or is the Zipfian pattern particularly amenable to recency-only eviction?
- Could adaptive or hybrid policies dynamically switch between eviction strategies depending on observed access patterns or hit-rate trends?

## Continuation

The cohort's uniform regression against ClassicLRU suggests this baseline is sound for the Zipfian workload and current cache geometry. Rather than pursuing further policy refinements in isolation, next cycle should: (1) test ClassicLRU's robustness across different workload distributions or cache sizes to identify scenarios where it breaks, (2) explore implementation-level optimizations (prefetching, batching, vectorization) that might reduce per-access overhead without changing policy, or (3) shift focus to scenarios (non-Zipfian access, cache contention, multi-core effects) where simpler eviction may falter. If frequency or finer buckets are revisited, prioritize measurement and optimization of implementation overhead as the primary constraint, not policy semantics.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cohort tested three orthogonal approaches to improve upon ClassicLRU: replacing item-movement bookkeeping with timestamp updates (TimestampLRU), minimizing bookkeeping entirely (FIFO), and consolidating cache geometry into wider, denser buckets (64 × 32). All three variants regressed on per-operation latency by 40–45%, while hit-rate either held steady or improved modestly (+0.24%). No variant achieved a fitness win over the baseline, revealing that ClassicLRU is not just a reasonable policy for this workload but a particulary well-optimized implementation of one.

## Patterns

- **Latency regressed uniformly despite different mechanisms**: Whether changing eviction policy (TimestampLRU, FIFO) or cache geometry (wider buckets), all variants slowed by ~40–45% on per-op time, suggesting ClassicLRU's vector operations on small buckets (16 items) have unusually good cache locality or compiler optimization that alternatives could not match.
- **Hit-rate insensitivity to policy details**: FIFO maintained 0.6611 hit-rate despite dropping recency awareness entirely, confirming that Zipfian access patterns alone can sustain high hit-rate without explicit recency tracking. TimestampLRU also held 0.6611, showing the bookkeeping method is orthogonal to hit-rate. Only geometry change (V002) moved hit-rate (+0.24%), but at a larger latency cost than policy changes.
- **Geometry trade-off: hash table vs. bucket scan**: Wider buckets (64 × 32) improved cache density and hit-rate but increased the cost of sequential intra-bucket searches, suggesting that for this workload, the latency penalty of scanning longer lists outweighs the benefit of fewer hash table lookups and better memory packing.
- **Theory-practice inversion deepens**: V001 (TimestampLRU) pursued O(1) timestamping vs. ClassicLRU's O(n) vector ops, yet achieved worse latency. This repeats the pattern from cycle 0001's Swap variant: asymptotic complexity gains evaporate under cache-level micro-architecture and implementation overhead.

## Open questions

- Why does ClassicLRU's O(n) vector manipulation (remove + append on 16-item vectors) outperform O(1) timestamp updates or zero-cost FIFO hits? Is the answer cache line coherency, branch prediction stability, or compiler loop optimization?
- Does the current geometry (128 × 16) happen to be locally optimal for this Zipfian workload and benchmark scale, or is it more broadly robust?
- Would batched or vectorized bucket scans recover the geometry advantage, or is the per-item cost of scanning inherently the bottleneck?

## Continuation

ClassicLRU appears not simply correct but finely optimized for this workload and hardware—attempts to improve hit-rate, reduce bookkeeping, or tune geometry all degraded latency without achieving a net fitness gain. Rather than continuing policy or geometry variations, the next cycle should: (1) understand *why* ClassicLRU is fast (profile micro-architectural effects, examine compiled code patterns), (2) test ClassicLRU's robustness across different workload distributions, cache sizes, or multi-core scenarios to identify conditions where it breaks, or (3) explore implementation-level refinements (prefetching, cache-aligned layouts, vectorization) that preserve ClassicLRU's performance profile while addressing potential bottlenecks outside the policy itself. If policy is revisited, focus on hybrid or adaptive strategies that can fall back to ClassicLRU's proven efficiency rather than replacement.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cohort tested the bounds of cache geometry: narrower buckets to reduce scan depth, a policy swap (FIFO) to avoid per-hit bookkeeping overhead at wider bucket sizes, and even wider buckets to push the hit-rate improvement trajectory. Narrower geometry (256×8) regressed both hit-rate and latency. The FIFO policy variant failed to execute its intended change, leaving the latency-improvement hypothesis untested. Wider geometry (32×64) extended hit-rate marginally but caused tail latencies to spike significantly, signaling an inflection point in the geometry trade-off space.

## Patterns

- **Geometry trade-off is saturating**: Cycle 0002's wider buckets (64×32) yielded hit-rate gain (+0.24%) at latency cost (-45%). Cycle 0003's even wider buckets (32×64) produced minimal hit-rate gain (+0.0005%) while tail latencies degraded sharply (p99 doubling), indicating diminishing returns and accelerating latency penalty as bucket width increases.
- **ClassicLRU's O(n) scan overhead is the bottleneck at scale**: Both v002 and v003 demonstrate that intra-bucket linear search becomes increasingly expensive as bucket size grows. The sharp tail-latency spike in v003 suggests that outlier scans on full buckets are dominating percentile performance, a micro-architectural constraint that narrower buckets avoid but at other costs.
- **Narrow buckets backfire despite theory**: The prediction that reducing scan depth (16→8 items) would improve latency was contradicted by v001's 45% regression. Hash-function distribution overhead and memory-layout effects across 256 buckets outweigh any benefit from shallower scans, repeating the pattern from prior cycles where theoretical asymptotic gains fail in practice.
- **Policy pivot was blocked**: H02's hypothesis—that FIFO's zero per-hit bookkeeping would decouple latency from bucket width—was sound but the variant either failed to compile the policy change or it was overridden. This leaves the testable claim unresolved and suggests a build/integration issue.

## Open questions

- Why did v002 (H02) not execute the FIFO policy change despite the hypothesis specifying it? Was this a build failure, override, or integration issue that could be fixed in the next cycle?
- If FIFO were correctly implemented on 64×32 geometry, would it flatten the latency cost while retaining the hit-rate gain, or does the policy change require additional structural changes?
- Is the tail-latency spike in v003 an inherent worst-case of linear bucket search, or an implementation artifact (e.g., cache misses, branch mispredictions) that could be mitigated?
- Has the cache geometry reached a local optimum, or does the fitness landscape favor wider buckets at different workload distributions or sizes?

## Continuation

The cohort has identified a clear frontier: wider buckets improve hit-rate but at an accelerating latency cost under ClassicLRU's per-hit movement operations. The next cycle should: (1) resolve the H02/FIFO integration issue and re-run v002 with the intended policy change to test whether decoupling latency from bucket width is feasible, (2) if FIFO succeeds, probe the local optimum around 64×32 FIFO to confirm it is a fitness win, or (3) if FIFO fails or shows no improvement, accept that ClassicLRU + 64×32 is the current Pareto frontier and shift focus to orthogonal avenues: robustness across different workload distributions, multi-core contention, or policies designed for large-bucket efficiency (e.g., segment-based or batched eviction).


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cohort pursued two competing strategies for managing wide-bucket latency: policy changes to reduce per-hit overhead (FIFO with timestamp victim selection, segment-based LRU), and implementation-level micro-optimizations (early-exit scanning and cache-line alignment) applied to ClassicLRU. The policy variants failed with latency regressions of 2×–4×. Only v003—which abandoned policy change and instead optimized the implementation—achieved a net improvement, preserving the 32×64 geometry's hit-rate gain (0.664) while halving tail latencies through early-exit and cache-aligned layouts.

## Patterns

- **Policy changes incur hidden per-operation costs that outweigh asymptotic gains**: Both v001 (FIFO) and v002 (segment-based LRU) suffered from changes that were theoretically sound but practically expensive. v001's timestamp-based victim selection replaced ClassicLRU's O(1) position lookup with O(n) bucket scan, adding 100–200 ns per operation. v002 added per-operation segment metadata maintenance that yielded nearly 4× latency regression despite bounding eviction scans. Both confirm the pattern from prior cycles: optimizing the eviction path must not burden the critical hit path.

- **Micro-optimization succeeds where policy fails**: v003 kept ClassicLRU's proven policy and geometry (32×64) but improved implementation details—early-exit on key match eliminates unnecessary bucket iterations, and 64-byte cache-line alignment reduces memory access latency. Result: 50% latency improvement on get operations and 33–50% on puts, with hit-rate preserved. This is the first variant to demonstrate that wide geometry's theoretical advantage (hit-rate) can be realized in practice through careful engineering.

- **Wide buckets are viable if latency is solved**: Cycle 0003 identified a latency crisis (p99 ≈300 ns) that made 32×64 geometry impractical. v003 proves the crisis was not geometric but implementational: address tail latencies through early-exit and cache alignment, and the geometry becomes a clear win—hit-rate +0.004 relative to ClassicLRU, latency returned to baseline levels.

- **Eviction optimization is a trap for high-occupancy buckets**: Both v001 and v002 optimized the eviction path, but at a bucket size (32–64 items), eviction is rare while hits are common. Spending per-hit cost to save infrequent eviction scans is a losing trade-off. v003 succeeds by optimizing the hot path (hit detection) instead.

## Open questions

- Is v003 a true Pareto win relative to ClassicLRU and all prior cycles? How does its fitness (hit-rate + latency) rank across the entire evolution history?
- Can v003's optimizations (early-exit, cache-line alignment) be backported to narrower bucket geometries, or are they specific to wide-bucket scanning efficiency?
- Does v003 maintain its advantage across different workload distributions, cache sizes, or multi-core contention scenarios, or is it optimized narrowly for the current Zipfian benchmark?
- Why does FIFO's victim selection require O(n) scan while ClassicLRU achieves O(1)? Could a data structure (e.g., linked list, heap, or O(1) min tracker) recover FIFO's zero-bookkeeping advantage without scan overhead?

## Continuation

The cohort demonstrates a critical insight: wide-bucket geometry is not inherently unviable, but requires micro-optimization to realize its hit-rate advantage without latency penalty. v003 is a turning point—it shows that the latency/hit-rate trade-off space has room for improvement through implementation cleverness rather than policy change.

Next cycle should: (1) confirm v003's fitness rank relative to the full evolution history and identify whether it is the current Pareto leader, (2) explore whether v003's optimizations (early-exit, cache-line alignment) generalize to other bucket sizes or narrow geometries, potentially unlocking further efficiency gains, (3) if v003 is the leader, probe its robustness across different workload distributions and cache sizes to identify its failure modes, or (4) if a larger policy innovation is desired, design approaches that prioritize hit-path efficiency—e.g., hybrid policies that fall back to ClassicLRU-like operations when micro-costs are high, or deferred/batched bookkeeping to amortize overhead.

Abandon further attempts to reduce per-hit bookkeeping overhead through policy isolation (v001, v002 both failed); the modern CPU memory hierarchy favors correct implementation of simple policies over complex policies with per-hit overhead.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cohort tested the generalizability of cache-line alignment and early-exit optimization, techniques that succeeded on 32×64 geometry in cycle 0004. Three variants applied these micro-optimizations to three different bucket geometries: baseline 128×16, parent 64×32, and ultra-wide 16×128. All three variants failed to improve fitness. Geometry-specific optimizations do not transfer across bucket configurations—a critical finding that challenges the assumption that micro-optimization is a general tool for latency control.

## Patterns

- **Micro-optimizations are geometry-sensitive, not universal**: v004/v003's success with early-exit and cache-line alignment on 32×64 was taken as a general principle in cycle 0005, but the results show it does not generalize. v001's application to baseline 128×16 actually *hurt* latency (get p50 +45%, p99 doubled), while v002's application to 64×32 produced no measurable gain. This reveals that the micro-optimization strategy was finely tuned to the specific bucket scan profile of 32×64, not a reusable optimization principle.

- **Micro-optimization saturates and inverts at geometric extremes**: v003's extension to ultra-wide 16×128 exposed the scalability limits. Early-exit and cache-line alignment were engineered to manage the cost of scanning ~30-item buckets, but cannot contain the cost of 128-item sequential searches. Put operations degraded 3×, get p99 doubled, and hit-rate declined due to tighter bucket distribution. The micro-optimization strategy inverts when bucket size exceeds its design threshold.

- **Compiler sophistication narrows the opportunity window**: v002's null result (no change in fitness despite adding manual optimizations to 64×32) suggests the Rust compiler is already applying equivalent optimizations at the assembly level or the workload pattern doesn't trigger scenarios where these hand-coded techniques provide measurable advantage. This pattern echoes cycle 0004's observation about modern CPUs: the margin for manual micro-optimization is shrinking.

- **Previous cycle's tentative winner may be a narrow local optimum**: Cycle 0004 positioned 32×64 with micro-ops as a turning point—the first variant to decouple latency from geometry. Cycle 0005's failure to generalize these optimizations suggests that v004/v003 was optimized specifically for that geometry and workload, not a foundation for further progress. The apparent "win" may not extend to other workload distributions, cache sizes, or access patterns.

## Open questions

- Is 32×64 with micro-ops truly superior to baseline 128×16 in fitness, or was cycle 0004's assessment overstated? Does this variant hold up under different workload distributions?
- Why does early-exit work on 32×64 but hurt on 128×16? Is the difference rooted in absolute bucket size, the interplay between hash-table lookups and intra-bucket scans, or cache-line occupancy patterns?
- Have we reached a fundamental limit where geometry and micro-optimization can no longer yield progress, requiring a return to policy or algorithmic innovation?
- Is the 32×64 point a true Pareto frontier, or a local optimum that becomes suboptimal under broader conditions?

## Continuation

Micro-optimization has reached a ceiling in this cycle—early-exit and cache-line alignment do not generalize across geometries and may have been over-fitted to 32×64. The cohort reveals that implementation-level tuning, while valuable for specific configurations, is not a scalable lever for overall fitness improvement. The next cycle should: (1) empirically re-evaluate whether 32×64 with micro-ops is truly a fitness win relative to baseline and prior cycles across multiple workload distributions before committing further optimization effort, (2) if micro-optimization is confirmed as locally optimal but not generalizable, return to orthogonal approaches—policy innovations designed for high-occupancy buckets, learned or adaptive placement strategies, or multi-core efficiency—rather than further geometric or implementation tuning, or (3) if micro-optimization is abandoned, consider broader architectural changes such as cache-oblivious designs or data structures (e.g., cuckoo hashing, learned indices) that might break the current geometry-latency trade-off entirely.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cohort tested three competing strategies to overcome the per-hit bookkeeping bottleneck identified in prior cycles: deferred eviction-time decision-making via access-distance epochs, micro-optimization through prefetch-aware reorganization, and adaptive hot/cold partitioning with dual indices. All three variants failed with either flat or regressed hit-rate and increased latency. This cycle represents the exhaustion of policy and structure-level optimization attempts within the current cache size and Zipfian workload context.

## Patterns

- **Eviction policy remains a non-lever across all strategies**: v001's epoch-based distance tracking, v003's hot/cold partitioning with frequency-based promotion, and v002's reorganization-aware reordering all employed fundamentally different policies, yet all produced flat or identical hit-rates (0.661–0.665, the Zipfian-indifferent baseline). Across cycles 0004–0006, this pattern repeats: hit-rate is invariant to policy selection. The conclusion that eviction strategy does not drive fitness on this workload is now confirmed beyond reasonable doubt.

- **Per-hit bookkeeping cost is structural and not addressable via indirection or deferral**: v001 attempted to move cost from hit path to eviction path (rare vs. common), but epoch tracking on every operation added more overhead than ClassicLRU's vector manipulation. v003 partitioned to isolate bookkeeping cost to a hot segment, but dual-index searches, frequency decay every 32 operations, and promotion logic added more latency than the savings from FIFO on cold items. The fundamental issue is not *where* bookkeeping happens, but that modern CPUs penalize any per-hit overhead, including tracking fields and conditional logic. Deferral or isolation does not eliminate the cost, merely distributes it.

- **Implementation complexity inverts good algorithmic ideas**: v003 was the most sophisticated variant algorithmically—adaptive hot/cold partitioning is a sound fit for Zipfian workloads where the working set is small and distinct. However, the implementation required dual index vectors, frequency tracking, decay loops, and promotion heuristics. The total per-operation cost of these mechanisms exceeded the benefit of using cheap FIFO on cold items. This mirrors cycle 0004's observation that policy innovation fails, but extends it: even well-reasoned structural changes fail when their implementation overhead is counted.

- **Metadata expansion adds cost without corresponding gain**: All three variants added per-item fields (epochs, frequency counters), expanding the Item struct by 20%. This overhead is paid on every operation but provides no improvement in hit-rate (which is policy-invariant). Cycle 0005 identified that compiler optimization narrows the margin for manual tuning; cycle 0006 shows that adding fields to enable algorithmic refinement backfires when the refinement itself doesn't improve fitness.

- **Cycle 0004's tentative success does not generalize outward**: v002 attempted to extend cycle 0004's v003 (32×64 + early-exit + cache-line alignment) by adding reorganization for prefetch coherency. Despite staying on the same geometry and preserving the existing optimizations, adding new logic (epoch tracking, in-place partitioning) disrupted the implicit ordering that early-exit relied on and added latency overhead that erased any prefetch gain. This suggests cycle 0004's v003 may have been over-fitted to a narrow point in the design space rather than a foundation for further progress.

## Open questions

- Is the cycle 0004 v003 (32×64 + micro-ops) truly a fitness leader relative to the full evolution history, or was it measurement noise from a narrow workload/cache-size window? Does it hold up under different access patterns, cache sizes, or contention scenarios?
- Have we exhausted the policy/geometry/micro-op space for single-threaded, fixed-size, Zipfian-workload caches? Is the baseline (128×16 ClassicLRU) Pareto-optimal at this level of analysis?
- Why does every attempt to add structure (tracking fields, index vectors, reorganization triggers) incur more cost than it saves, despite sound algorithmic motivation? Is this a fundamental property of CPU memory hierarchies, or are we missing a architectural insight?
- Should the next cycle pivot to orthogonal directions (learned strategies, alternative data structures, multi-core efficiency, robustness across workload distributions), or first empirically validate that cycle 0004 v003 is truly superior and worth defending?

## Continuation

Cycle 0006 has hit a ceiling: per-hit bookkeeping cost is structural, not addressable via policy, geometry, or micro-optimization. Every "clever" idea (deferral, partitioning, prefetch awareness) adds implementation cost that exceeds benefit. The next cycle must choose between two paths:

**Path A (Robustness & Validation)**: Empirically confirm whether cycle 0004's v003 is a true fitness leader by testing it across different workload distributions, cache sizes, and contention scenarios. If it holds, shift focus to understanding its failure modes and robustness boundaries rather than pursuing further optimization.

**Path B (Pivot to Orthogonal Directions)**: Accept that single-threaded, Zipfian, fixed-size cache optimization is saturated. Pursue fundamentally different approaches: learned insertion strategies (e.g., based on access patterns or key properties), cache-oblivious designs that break geometric dependence, cuckoo hashing or other probing strategies to reduce per-hit bookkeeping, or multi-core efficiency and contention reduction.

Recommend Path A first to establish a firm baseline, then Path B if robustness testing reveals brittleness. Do not continue policy or micro-optimization refinement—all three variants confirm the marginal return is zero or negative.


### Cycle 0007

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cohort tested three orthogonal refinements to the prior winner (cycle 0004 v003): geometry scaling to validate generalizability, tail-preferential eviction to improve hit-rate without overhead, and learned insertion positioning based on access history. All three variants failed to improve net fitness—geometry scaling severely degraded latency, naive recency tracking reduced hit-rate, and the learned approach achieved modest hit-rate gain at prohibitive latency cost (40–50% increase).

## Patterns

- **Cycle 0004 v003's optimizations are finely tuned to 32×64 geometry, not generalizable**: v001's attempt to change bucket geometry from 32×64 to 64×32 (narrower buckets) caused severe latency regression—get p99 doubled, put p50 tripled. Early-exit optimization's effectiveness is inseparable from wide buckets; the cost-benefit of scanning fewer items diminishes at smaller bucket sizes. This confirms that the prior winner is a localized optimum rather than a foundation for proportional scaling.

- **Simple recency heuristics fail to capture Zipfian reuse distances**: v002's tail-preferential eviction with 64-operation recency reset intervals achieved flat latency but reduced hit-rate significantly (0.664 → 0.6137). The coarse-grained recency flag is not a sufficient proxy for the frequency/reuse-distance patterns in the Zipfian workload. Position-based LRU ordering implicitly captures these patterns better than explicit recency tracking.

- **Learned insertion positioning has algorithmic merit but faces implementation overhead barriers**: v003 improved hit-rate by +0.43% through predictive bias based on recent access history, proving that algorithmic adaptivity (learning from temporal locality shifts) can steer cache behavior positively. However, polling a 16-element rolling buffer on every operation incurred 40–50% median latency overhead, making the hit-rate gain Pareto-dominated by latency cost. The idea is sound; the implementation is inefficient.

- **No variant achieved net fitness improvement; prior winner remains unbeaten**: Despite three distinct approaches—scaling, eviction-policy refinement, and learned insertion—all produced either flat or regressed fitness. Cycle 0004 v003 retains the best fitness across all eight cycles and appears to be a genuine local optimum in the current design space.

- **Metadata expansion and per-operation bookkeeping remain liabilities**: v002 added 8-byte per-item boolean fields; v003 added cache-struct overhead. Both incurred costs without commensurate gain. This reinforces cycles 0006–0007 findings that any structural change to enable algorithmic refinement tends to cost more than it saves, especially on latency-sensitive paths.

## Open questions

- Is v003's learned insertion approach viable with an amortized or vectorized implementation that reduces per-operation polling overhead to acceptable levels?
- Does cycle 0004 v003 remain optimal across different workload distributions (non-Zipfian, multi-modal access patterns) and cache sizes, or is its fitness edge workload-specific?
- Have we reached the ceiling of single-threaded, fixed-size cache optimization, or are there unexamined algorithmic directions (e.g., adaptive learned thresholds for eviction, cache-oblivious probe ordering)?
- Is the 0.43% hit-rate improvement from learned insertion worth pursuing if overhead can be reduced, or does the Zipfian workload's inherent hit-rate ceiling (empirically ~0.7 across variants) limit upside?

## Continuation

Cycle 0008 closes the refinement and scaling paths for cycle 0004 v003: geometry scaling fails, simple eviction-bias refinement fails, and learned insertion trades unacceptable latency for marginal hit-rate. The cohort confirms that the prior winner is a precise local optimum that resists incremental improvement.

**Recommend two parallel investigations for the next cycle:**

1. **Robustness validation of cycle 0004 v003**: Test its fitness across non-Zipfian workload distributions (uniform, multi-modal, temporal locality shifts), varying cache sizes (512–8192 entries), and contention scenarios. If it remains superior, the design space is saturated and focus should shift to implementation stability and generalization; if it regresses, the next cycle can pursue alternatives with confidence.

2. **Lower-overhead learned insertion**: If v003's algorithmic idea of predictive biasing is to be explored further, redesign to amortize buffering cost (e.g., update insertion bias every N insertion operations rather than every operation, or vectorize history tracking). A 0.43% hit-rate gain is modest but not negligible if overhead can be halved or quartered.

If neither direction yields improvement, the next cycle should pivot decisively to orthogonal approaches: alternative data structures (cuckoo hashing, Robin Hood variants with amortization), cache-oblivious designs, multi-core efficiency, or learned indexing strategies that operate at insertion time rather than in-path polling.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cohort pursued three strategies to address the fundamental trade-off identified in cycle 0008: hit-rate gains are algorithmically achievable but latency-expensive. v001 attempted to amortize overhead via deferred buffer updates; v002 pursued minimal-cost frequency metadata via probabilistic counters; v003 validated that workload characterization can significantly improve hit-rate (+8.1%) through adaptive insertion bias. All three failed to achieve net fitness improvement. v003 succeeded algorithmically but at latency cost (2–3×) that exceeds benefit; v001 and v002 failed to reduce overhead while maintaining performance.

## Patterns

- **Latency is the immovable constraint in single-threaded, Zipfian cache design**: All three variants hit latency walls despite different intervention points. v001's amortization created p99 spikes (get p99 doubled to 200ns) rather than smoothing cost; v002's probabilistic frequency tracking added ~31ns to the hot get path despite minimal memory overhead; v003 achieved the largest hit-rate improvement of the cycle (+8.1%) but at cost of get p99 increasing from 100ns to 300ns. Across all variants and all prior cycles, latency degradation accompanies any algorithmic gain, suggesting that modern CPU hierarchies penalize any per-operation bookkeeping—whether deterministic (v003's insertion-position logic) or probabilistic (v002's Morris counter)—more than they benefit from improved hit-rate.

- **Characterization-based insertion bias works algorithmically but violates its own hypothesis**: v003 proved that analyzing access patterns during an initial phase (512 operations) can identify temporal locality and steer insertion bias accordingly, yielding +8.1% hit-rate improvement. However, the hypothesis assumed "zero per-operation overhead during normal execution," which proved false. The insertion_bias logic runs on every put operation (not just during characterization), and tracking fields add 56 bytes of persistent overhead. This reveals a false dichotomy: characterization can be one-time (but then it cannot adapt to workload shifts), or continuous (but then it always costs per-operation overhead). v003 attempted both and paid both costs.

- **Amortization does not smoothen latency spikes; it consolidates them**: v001's core premise was sound—batch buffer updates every 32 operations to reduce per-operation cost from 40% down to ~1–2%. The result was the opposite of the hypothesis: p50 latency remained flat, but p99 latency doubled, indicating that deferral creates load imbalance at the operation-granularity where it matters most. Buffering overhead until specific operations causes intermittent expensive operations (the 32nd, 64th, …), failing the requirement that tail latency remain acceptable. This pattern (amortization backfires on latency-sensitive paths) is new to cycle 0009 and suggests amortization is fundamentally incompatible with per-operation latency constraints.

- **Minimal-cost frequency metadata still incurs unacceptable cost on the hot path**: v002 reduced per-item frequency tracking from 8+ bytes (explicit counters) to 1 byte (Morris counter), cleverly reusing struct padding. However, the probabilistic increment logic in the hot get path—seed computation, masking, conditional branch—added ~31ns per access, a 31% latency cost with no hit-rate improvement. The result suggests that this workload lacks clear hot/cold access patterns that frequency-aware eviction exploits, or that any frequency-aware victim selection is latency-infeasible for this cache design.

- **Hit-rate gains consistently exceed latency budgets**: Across cycle 0008 (v003: +0.43% hit-rate at 40–50% latency cost) and cycle 0009 (v003: +8.1% hit-rate at 2–3× latency cost), algorithmic improvements to hit-rate are Pareto-dominated by latency cost. Even when a variant improves hit-rate by double-digit percentages, the per-operation latency cost is so high that no workload can afford to use it. This suggests the current design space has reached a Pareto frontier where hit-rate and latency are tightly coupled, and improving one unavoidably degrades the other.

- **Metadata expansion is inescapable and non-negotiable**: Every variant attempted different strategies to control metadata bloat. v001 added 16 bytes for amortization counter; v002 added 1 byte per item; v003 added 56 bytes of tracking fields. None yielded net fitness improvement because the latency penalties exceeded any benefit. This reinforces cycles 0006–0007 findings that structural changes to enable algorithmic refinement invariably cost more in common cases than they save.

## Open questions

- Is v003's +8.1% hit-rate improvement achievable at lower latency through alternative implementations (e.g., vectorized history tracking, pre-computed bias classes, amortized bias recalculation)?
- Why does amortization consolidate overhead into operation spikes rather than smoothing it? Is there a batching or vectorization strategy that can achieve true amortization without tail-latency regression?
- Does v002's frequency-aware approach only fail because this workload lacks hot/cold boundaries, or is probabilistic frequency tracking fundamentally latency-incompatible with this cache design?
- Should the next cycle pursue non-Zipfian workloads where the Pareto frontier might be different, or commit to latency as the binding constraint and validate prior cycle winners (0004 v003, 0006 v003) across robustness scenarios?

## Continuation

Cycle 0009 confirms cycle 0008's saturation finding: within the Zipfian, single-threaded, fixed-size cache space, hit-rate improvements are consistently Pareto-dominated by latency cost. v003's +8.1% hit-rate gain is algorithmically validated but latency-infeasible; v001 and v002 failed to reduce overhead while maintaining performance. The cohort reveals two additional insights: (1) amortization consolidates overhead into spikes rather than smoothing it across operations, undercutting a key hypothesis; (2) even 1-byte per-item metadata can incur unacceptable hot-path latency cost if the logic is not transparent to compiler optimization.

**Recommend the next cycle pursue one of two directions:**

**Path A (Latency as binding constraint)**: Accept that hit-rate and latency are Pareto-coupled, and single-threaded optimization is saturated. Validate whether cycle 0004 v003 or cycle 0006 v003 is the true fitness leader across different workload distributions (uniform, multi-modal, temporal shifts) and cache sizes to establish robustness baselines. Shift focus from optimization to failure-mode identification: when and why do the current leaders break?

**Path B (Escape the Zipfian frontier)**: If latency-tolerant workloads or non-Zipfian access patterns exist where hit-rate improvement is worth cost, test v003's characterization approach on those workloads. Or pursue vectorization/batching to implement hit-rate-improving strategies with lower per-operation latency (e.g., batch characterization recalculation every N operations instead of continuous, or vectorize history buffer updates). Do not continue refinement of amortization, frequency tracking, or per-operation deferral within the Zipfian, single-threaded space—all three are empirically closed in this cycle.


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
