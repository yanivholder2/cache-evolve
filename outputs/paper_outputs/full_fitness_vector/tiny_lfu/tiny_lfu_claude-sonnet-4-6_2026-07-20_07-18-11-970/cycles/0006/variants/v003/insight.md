# Variant v003 Insight

## Helped

_(none — no dimension improved over the parent)_

## Hurt

- `hit-rate`: Deriving all four row positions from a single SipHash root via the remix chain reintroduced inter-row correlation that two independently seeded hashes eliminated; the sketch's frequency estimates became noisier, causing the admission filter to make slightly worse eviction decisions.
- `per-op-time`: Replacing one independent `seeded_hash` call with three chained `remix` steps created a sequential multiply→rotate→XOR dependency that the CPU cannot overlap; the resulting instruction-level serialisation cost more p50 latency than the removed SipHash invocation saved.

## Takeaway

Switching from two seeded SipHash calls to a single-hash-plus-remix chain neither preserved hit rate nor improved throughput: the deterministic bijective derivation shares a common hash root across all four row positions, producing enough correlated collisions in the count-min sketch to measurably lower hit rate, while the sequential `wrapping_mul` dependency chain across three remix steps stalls out-of-order execution more than a second independent SipHash invocation did. True seed independence, not bijective width, is what made the parent's two-hash design accurate.
