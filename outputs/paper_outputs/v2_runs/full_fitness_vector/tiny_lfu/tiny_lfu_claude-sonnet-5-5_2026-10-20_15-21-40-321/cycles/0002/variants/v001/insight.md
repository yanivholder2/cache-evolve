# Variant v001 Insight

Note: I could not list or read the overlay source in this session. This insight comes from the hypothesis and the two fitness vectors, not from a code diff.

## Helped

- per-op-time: The change computes one splitmix64 finalizer per operation and takes the row indexes from bit-slices of it with a power-of-two mask. That removes the remaining per-row mixing and the modulo from `increment` and `estimate`. Both `get` and `put` got faster, and the `put` tail improved most. `put` is the op that calls the sketch three times on a full cache. Caching the hash from `get` for the following `put` on a miss probably adds to this.

## Hurt

- metadata-size: The `TinyLfuCache` struct grew slightly, consistent with the added cached-hash state. `Node` stayed at its constraint of 48 bytes and the sketch heap did not change. The dimension summary tracks `Node`, so the growth is invisible there and shows only in the per-type breakdown.

## Takeaway

After v003 had already cut SipHash to a single hash, the remaining per-op cost was still in index derivation and recomputation. Replacing it with one strong mix and mask slicing, and reusing the hash across `get` then `put`, gave a clear latency gain at no cost to admission quality. Hit-rate is essentially unchanged because the counters, conservative update and admission rule were left alone. The price was a few bytes of cache-struct state for the cached hash.
