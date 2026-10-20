# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0003

## Winners

- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_05-10-08-572/cycles/0003/hypotheses/H03.md

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
