# Variant v003 Insight

## Helped

None. hit-rate landed exactly on the parent's value, as the hypothesis required. That confirms the in-place ghost-hit branch, the mask-based `take_ghost` and the MRU early return keep the parent's behaviour; the side-by-side reference test checks this too. metadata-size is unchanged because neither `Item` nor the cache header was touched. put p99 held at the parent's level, which means removing the two whole-bucket memmoves (`evict_front()` → `remove_at(0)` plus `insert(len0)`) on full ghost-hit puts did not trim the put tail in any visible way.

## Hurt

- per-op-time (get): Only one change touches the get path: the new `pos == items.len() - 1` early return in `hit()`. It adds a compare and branch to every protected hit. With 15/16 of a 16–32-slot bucket protected, a hit lands exactly on the MRU slot less often than the hypothesis assumed, so the branch rarely saves a rotate and is a likely source of mispredicts. That fits the widening get tail more than the median.
- per-op-time (put): `take_ghost` now always scans a full chunk with no early exit. The ring is 8–16 entries, so on a ghost hit it does the whole ring where `position()` would have stopped at the match. The new `ghost_hit && full && len0 > 0` dispatch adds a branch ahead of every cold insert. Meanwhile the memmove it removes is only a few hundred bytes, and only on the subset of puts that are full-bucket ghost hits. The added fixed work on every put outweighed the movement it saved.

## Takeaway

On this bucket size the miss-path memmove was not a meaningful cost. Two whole-bucket shifts over 16–32 sixteen-byte items are cheap, and full ghost-hit puts are a minority of puts, so even fully removing them leaves nothing to measure. The micro-optimisations that came with it add fixed per-op work: a branch-free ghost scan that gives up early exit, and an MRU short-circuit that rarely fires under a 15/16 protected segment. That work made both the get and put medians slightly worse, and the get tail clearly worse. Data-movement wins in this lineage only pay off when the moved range is large or on the hot hit path, not on the rarely taken full-ghost-hit miss branch.
