# Variant v003 Insight

## Helped

- hit-rate: The admission policy is unchanged (same LRU movement, strict-greater contest, depth 4, same width), so the only thing that changed is which keys share counters. One FxHash+splitmix64 hash split into four rows by Kirsch–Mitzenmacher double hashing gives collisions at least as good as four separate SipHash rows. The small gain probably reflects a different collision pattern on this trace, not a better policy.

## Hurt

## Takeaway

On hit-rate, swapping four per-row SipHash passes for one fast 64-bit hash and double hashing is safe. Four sketch rows built from one hash were enough to keep TinyLfu's frequency contest accurate on workload-a. This variant's evaluation produced only hit-rate, with no per-op-time or metadata-size entries. So the claim this hypothesis was actually about, lower get latency from one hash per key, flat counters and a fast map hasher, is still untested. v003 should be re-benchmarked before the speedup is credited. By design the sketch geometry is unchanged. Swapping the map's hasher to `BuildHasherDefault` (which has no size) should, if anything, shrink the cache struct rather than grow it.
