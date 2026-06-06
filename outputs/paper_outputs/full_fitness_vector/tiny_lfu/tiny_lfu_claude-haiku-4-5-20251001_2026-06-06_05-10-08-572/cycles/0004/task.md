# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0004

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0004/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0004/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0004/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0004/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0004/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
