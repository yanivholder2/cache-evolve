# Cycle 0006 Insight

## Cohort outcome

All three variants branched from cycle 0005's Fx-hashed S3-FIFO. v001 widened the small queue, v002 ported the packed `u32` side array with key swap, and v003 made the ghost 2-way associative and twice as large. All three are listed as Winners, but none advanced hit-rate. v002 is the only real gain: a smaller per-node footprint with the same eviction order, paid for with slower `get`. v001 and v003 both lost a little hit-rate and slowed `put`.

## Patterns

- On workload-a, S3-FIFO's admission knobs appear to be at or near a local optimum, as SIEVE's in-queue knobs were in cycle 0003. A longer probation window took space from main, and extra ghost recall pushed long-reuse keys into main. Both changes cost more than they recovered. Main capacity for proven-reuse keys is the scarce resource.
- The bounded reach of the direct-mapped ghost was acting as a useful filter. Collisions limited how far back a "returning" key could be and still skip probation. Ghost tuning should shorten reach, not lengthen it.
- The side array's latency effect depends on the policy. On SIEVE it helped the `put` tail. On S3-FIFO's heavier miss path, that benefit disappears, and `get` now touches two cache lines (flags and value) instead of one. Representation changes still preserve eviction order safely, but they are no longer free on latency.
- The `put` path on S3-FIFO is now dominated by hashing and ghost probes on each miss. Changes that add ghost work show up at the `put` tail right away.
- Metadata accounting is still blind to heap side structures: the larger ghost and the `links` buffer and `Vec` header are not counted. The per-node wins overstate the real savings.

## Open questions

- Can VISITED stay inline with the value while only the links move out, keeping most of the memory win without the `get` penalty?
- Would a smaller ghost, sized to the small queue or to main's reuse distance, or a softer admission for ghost hits (e.g. entering main unvisited behind the hand) beat the cycle-0005 configuration?
- Does any of this hold at another cache size or trace? This has been unverified for five cycles, and the lineage is now tuning against a single point.

## Continuation

Keep cycle 0005 v003 as the hit-rate parent. Use cycle 0006 v002 only where memory matters, and try a variant that keeps VISITED next to the value. Drop wider small queues and larger or more associative ghosts. If ghost tuning continues, go smaller or make ghost admission less aggressive, and test split points just around 10% (1/8, 1/12) rather than wider. Before any further knob tuning, evaluate at a second cache size or trace, and count ghost and side-array heap bytes when judging metadata.
