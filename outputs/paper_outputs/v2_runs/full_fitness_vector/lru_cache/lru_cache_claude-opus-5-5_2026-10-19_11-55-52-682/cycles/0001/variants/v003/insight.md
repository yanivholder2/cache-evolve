# Variant v003 Insight

## Helped

- hit-rate: When the cache is full, new keys now go in at the head (LRU end), with only 1 in 32 going in at the MRU end. A key that is never re-referenced is evicted on the very next miss, so the looping workload-a working set no longer pushes out the resident set on every pass. Most of the cache stays fixed and keeps hitting on each loop instead of thrashing as plain LRU does. This confirms H03's claim at size 2048.
- per-op-time: Both get and put got faster at p50 and p99 even though the code path is not shorter (put gains a counter increment and a branch, and get is unchanged). The likely cause is memory access: on most misses the slot just filled at the head is the next one reused, so the same few arena nodes and list links take most of the churn and stay in CPU cache. With more hits there are also fewer evictions, and each eviction costs a key clone plus a HashMap remove and insert. This is an inference: the code does not show the cause directly, and some of the gain may be benchmark noise.

## Hurt

- metadata-size: The per-entry `Node` layout is unchanged. The `LruCache` struct grew by one 8-byte word because the `insert_tick: u32` counter is padded. This is a fixed cost per cache, not per entry, so the dimension's summary does not move.

## Takeaway

On this looping trace, changing only where a new key is inserted is enough to gain loop resistance. Adding BIP's head insertion to an otherwise unchanged LRU cost no extra per-node state, needed no change to how keys move on a hit, and added only one counter. The 1-in-32 MRU insert and the warm-up rule that fills the cache in plain LRU order were enough to keep the resident set useful, so later variants should try policies with segments or per-node state on top of BIP rather than replacing it.
