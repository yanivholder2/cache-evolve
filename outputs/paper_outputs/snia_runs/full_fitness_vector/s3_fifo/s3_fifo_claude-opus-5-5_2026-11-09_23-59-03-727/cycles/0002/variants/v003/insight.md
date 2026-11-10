# Variant v003 Insight

## Helped

- per-op-time: `get` now does one `index` hash lookup and a `slab` access, where v002 scanned `small` and then `main` with `find`. This is the cause of the lower `get` median and of its tighter tail.
- per-op-time: `put` gained the most, mainly at p99. v002 paid for a resident scan plus a full `ghost.iter().any` and `retain` on every miss. v003 replaces all three with hash operations (`index.get` and `ghost_index.remove`). Promotions and second-chance rotations now move a `u32` slot index instead of a whole `Entry`. The cost of keeping stale ghost tuples is one `retain` compaction after about `main_size` pushes, spread across those pushes.

## Hurt

- metadata-size: The per-entry `Entry<u64,u64>` footprint, which is the summary metric, did not move. The fixed `S3Fifo<u64,u64>` struct grew, though, because it now holds the slab `Vec`, the `free` list, two `HashMap`s and `ghost_seq`. The per-key memory these structures allocate is not counted in this dimension either: the resident index, the ghost index, and the ghost deque, which can hold up to twice `main_size` tuples.

## Takeaway

Swapping the linear scans for a slab with a resident hash index and a lazily deleted ghost index produces exactly the same hit rates as v002 on every trace and size. That confirms the policy decisions were preserved, including the live-ghost bound counted as `ghost_index.len()`. So the per-op-time gain can be credited to the data-structure change itself, unlike the timing shift in v002, which was put down to the environment. The cost lands in memory that metadata-size does not measure: extra per-key hash-map and ghost-tuple storage. Future variants that need a memory comparison should measure heap bytes per resident key.
