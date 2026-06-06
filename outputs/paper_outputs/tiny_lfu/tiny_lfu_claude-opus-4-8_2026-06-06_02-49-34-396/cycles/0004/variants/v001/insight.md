# Variant v001 Insight

## Helped

- metadata-size: Replacing `prev`/`next` (and the cache's `head`/`tail`) from `Option<usize>` to a `u32` with a `NIL = u32::MAX` sentinel halves `Node<u64,u64>` and shrinks the cache struct, since each link drops from 8 bytes to 4 and the enum discriminant/padding the `Option<usize>` carried disappears.
- per-op-time: The `!= NIL` branch tests with `as usize` indexing are slightly cheaper and more predictable than `match`-on-`Option` on the unlink/push_back/eviction-walk hot paths, trimming both `get` and `put` tail (p99) latency.

## Hurt

- (none): No dimension regressed; the smaller node layout left the eviction and admission logic byte-for-byte identical.

## Takeaway

Folding v002's `u32`-sentinel link encoding onto v003 is a free win: it cuts the per-node footprint in half without touching the hashing, sketch, or sampled-eviction semantics, so hit-rate is bit-identical while tail latency even improves slightly from the simpler branch tests. Compact integer links with a sentinel strictly dominate `Option<usize>` for an index-based intrusive list.
