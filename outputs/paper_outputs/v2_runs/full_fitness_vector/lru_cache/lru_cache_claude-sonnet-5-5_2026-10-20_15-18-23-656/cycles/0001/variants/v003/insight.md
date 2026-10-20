# Variant v003 Insight

## Helped

- per-op-time: The cheap multiplicative FxHasher replaces SipHash on every `map.get`/`insert`/`remove`, and `touch` skips relinking when the node is already the tail. Together these cut p50 for both ops, with the bigger drop on `put`, which hashes up to three times on the eviction path.
- metadata-size: `LruCache` itself got smaller because `BuildHasherDefault<FxHasher>` is zero-sized, where the default `RandomState` carries two u64 keys. `Node` size is unchanged.

## Hurt

- per-op-time: The `get` p99 tail got much worse, even though p50 improved. A weak, non-randomized hash with a simple high-to-low fold likely clusters some keys into long probe sequences on the trace, so a few lookups become slow. The no-op fast path in `touch` does not affect tail latency.

## Takeaway

Swapping in an inline Fx-style hasher and short-circuiting MRU re-touches lowers median latency and shrinks the struct, and it leaves hit-rate untouched because LRU order is preserved. The cost is a worse tail, so hash quality matters as much as hash speed. The planned removal of the eviction-path key clone was not fully landed: `put` still clones the key there, so some of the available `put` savings were left unused.
