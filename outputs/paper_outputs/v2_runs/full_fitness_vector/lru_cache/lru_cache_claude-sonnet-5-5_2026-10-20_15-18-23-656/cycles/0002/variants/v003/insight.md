# Variant v003 Insight

## Helped

- metadata-size: Replacing the `usize`/`Option<usize>` prev/next links with `u32` indices (`u32::MAX` as NIL) shrinks `Node` and the cache struct. The `Option` discriminant padding is gone and each link is half as wide.
- per-op-time: Smaller nodes put more of the arena in each cache line. The head-already-front early return skips relinking when the touched entry is already MRU. The Fx multiply plus fold hasher is also cheaper than the baseline hasher. Together these cut both median and tail latency for get and put.

## Hurt

- No dimension regressed. The sentinel-index and hasher changes did not alter eviction order, so hit-rate stayed the same.

## Takeaway

Narrowing the arena links to `u32` with a sentinel is a pure layout win: it shrinks metadata and speeds up ops without touching policy, so hit-rate is unchanged. The avalanche fold on the Fx hash avoided the p99 tail regression that weak low bits had caused in an earlier attempt. The cost is a hard capacity ceiling of `u32::MAX - 1`. I could not read the source diff in this session, so the attribution of the speedups across the three changes (links, hasher, early return) is inferred from the hypothesis rather than measured.
