# Variant v003 Insight

## Helped

(none: no dimension improved over the parent)

## Hurt

- hit-rate: Hit-rate did not rise, which rules out the hypothesis that ghost collisions were what held back S3-FIFO's gain. The ghost now holds two fingerprints per set across `next_power_of_two(capacity)` sets, so it remembers about twice as many keys as the cache holds. The small queue that feeds it is only 10% of capacity. The extra fingerprints it keeps are mostly long-reuse-distance keys. A ghost hit now sends these keys straight to the main tail, unvisited, where they push out SIEVE residents with better reuse. The extra ghost recall therefore turns into extra main-queue pressure rather than extra hits, and it slightly erodes the parent's gain.
- per-op-time: `put` slowed at both p50 and p99, against the hypothesis's "flat" prediction. Doubling the ghost doubles the working set of a randomly indexed table that every miss probes. `ghost_insert` is now a read-modify-write of two words (shift way 0 into way 1) instead of one store. `get` never touches the ghost, and its p50 is essentially unchanged. Its worse tail is most plausibly cache pollution from the larger ghost, or noise, not a code-path change.
- metadata-size: The measured summary did not move (`Node` is still 24 bytes and the `LruCache` struct is the same). This hides a real cost: the ghost heap buffer doubled, about 8 KiB more at capacity 2048. No current dimension counts it.

## Takeaway

The ghost's recall was not the bottleneck: keeping colliding fingerprints with 2-way associativity and 2× the slots made hit-rate slightly worse and `put` slower. On workload-a, the direct-mapped ghost's collision losses were acting as a useful filter: they bounded how far back a "returning" key could be and still skip probation. Future ghost tuning should go the other way, toward a ghost sized relative to the small queue or main-queue reuse distance rather than to full capacity, or toward making ghost-admitted keys enter main less aggressively.
