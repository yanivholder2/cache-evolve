# Variant v003 Insight

## Helped

- put per-op-time: put p50 and p99 moved slightly in the favorable direction. This is probably noise or a side effect of cheaper tail shifts on the shared bucket path, because the hypothesis never touched the put code.

## Hurt

- per-op-time (get): the get p50 got slightly worse and the p99 slightly better, so the change did not deliver the targeted speedup. Baseline `Vec::remove` followed by `push` already compiles to roughly one memmove of the tail. `rotate_left(1)` on a bucket of a few 40-byte items does about the same work, plus a read and write of the rotated element, so there was nothing to save.

## Takeaway

Replacing `Vec::remove` + `push` with `items[position..].rotate_left(1)` and an early return for the already-last item preserved ClassicLRU's ordering exactly, so hit-rate and metadata-size did not move. The speedup did not appear, because the hit path in small buckets is dominated by hashing, lookup and dynamic dispatch, not by how the item is moved. The p50/p99 differences are within measurement noise, so micro-optimizing the move itself is a dead end. Later attempts at get latency should look at lookup or dispatch costs instead.
