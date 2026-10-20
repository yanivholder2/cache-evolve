# Variant v003 Insight

## Helped

- per-op-time: replacing SipHash with the inline FxHash-style single xor+multiply (`fold`) on the u64 key map cut both get and put p50, and sharply lowered put p99.
- metadata-size: the zero-sized `FastBuildHasher` drops the default `RandomState`'s two-u64 seed, shrinking the `LruCache` struct's footprint — an effect the hypothesis predicted would be neutral.

## Hurt

- per-op-time: get p99 regressed; the cheaper hash lowered the median but the get tail latency moved the wrong way.

## Takeaway

Swapping SipHash for a one-multiply integer hasher is a modest but real median win on 8-byte keys, and the switch to a zero-sized build hasher is a free side win: it removes `RandomState`'s 16-byte seed from the struct. The cost is a slightly worse get tail latency, so the hashing change is not strictly dominant on per-op-time.
