# Variant v002 Insight

## Helped

(none — no dimension improved; metadata-size did not move, since `Item` and the cache header are unchanged and the bucket only swaps `ghost_head` for `ghost_mask`, which is the same size)

## Hurt

- hit-rate: The direct-mapped table keeps the ring's capacity but changes how ghosts age out. With the ring, a ghost always lasted exactly `ghost_len` evictions. With the table, it lasts until another evicted tag maps to the same `tag & ghost_mask` slot. Some ghosts get overwritten after one or two evictions, and others outlive the recency window. Losing a ghost early turns a re-put into a cold key at probation LRU instead of a ghost hit at probation MRU. That explains the small drop, which stays within the hypothesis's 0.001 tolerance.
- per-op-time (put): The median did not improve and the tail got worse, so removing the `position()` scan saved nothing measurable. A scan over 4–16 `u16` slots in one or two cache lines was already cheap, probably vectorized. The miss-path put cost is dominated by `find_tagged` and by the `remove(0)` / `insert(len0)` shifts on ghost hits, and this change touches neither.
- per-op-time (get): The tail rose even though the get path is byte-for-byte the same as the parent's (`find_tagged` + `hit`). Treat this as run-to-run noise or a code-layout effect, not a real cost of the table. That also means the put-tail rise can't be blamed on the table with confidence. The safe conclusion is that put latency didn't improve.

## Takeaway

Swapping the linear ghost scan for an O(1) direct-mapped lookup does not pay off at these ghost-store sizes. The ring scan was never the put bottleneck, so the "non-linear ghost lookup" question is answered: it isn't worth pursuing. Meanwhile, collision-based aging gives up a little of the ring's exact FIFO recency window and costs hit rate. Cutting put latency further means attacking the element shifts on the ghost-hit and eviction paths, not the ghost lookup.
