# Variant v002 Insight

## Hurt

- **hit-rate**: XOR-combined hashing failed to reduce collision-induced estimation errors; the workload either lacks collision patterns that benefit from decorrelation, or the specific hash combination strategy (two DefaultHasher instances seeded differently) does not decorrelate effectively in practice.
- **per-op-time**: Composite hashing requires two full hash computations and an XOR per position lookup; get operations slowed ~36% and put operations ~27%, making the approach computationally expensive despite the cache being small.

## Helped

(none)

## Takeaway

Decorrelating hash functions is theoretically sound for count-min sketches, but this implementation of XOR-combined independent hashes provided no hit-rate benefit while imposing a substantial latency cost. The hypothesis's core assumption—that correlated collisions were limiting accuracy—did not hold for this workload, or the specific decorrelation strategy (two seeded DefaultHasher variants) is inadequate. Direct correlation reduction via hash function design appears to be a wrong lever for hit-rate improvement in this cache.
