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
