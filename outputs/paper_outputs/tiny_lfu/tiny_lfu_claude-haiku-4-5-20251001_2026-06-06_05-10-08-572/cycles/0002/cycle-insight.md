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
