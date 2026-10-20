# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0006

## Winners

- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
