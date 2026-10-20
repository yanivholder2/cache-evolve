# Variant v003 Insight

## Helped

- per-op-time: get and put both measured faster at p50 and p99, but the code change does not explain it. `pair_get`, `find` and `hit()` are byte-for-byte the same as v002's. `pair_put` now does *more* work on every miss: one or two `hash()` calls for the fingerprints, plus a linear scan of a `max_size`-entry `u16` ghost ring. A ghost hit also skips the `items.insert(len0, ..)` shift by doing a `push`, but that is too rare to move get latency. Treat this movement as benchmark variance, or as a side effect of layout or allocation, not as a gain from the ghost list.

## Hurt

- hit-rate: hit rate fell below v002's, so the claim failed. Each ghost hit puts the re-admitted key straight into protected MRU. Because protected is usually full at its 3/4 cap, that push demotes protected's LRU item (`len0 += 1`) into probation, where the next miss can evict it. So a key with exactly one recent re-reference, seen only through its ghost, displaces an item that earned protection with a real second hit. The ring holds only `max_size` fingerprints and records only probation victims. It therefore remembers a reuse window of roughly one bucket's worth of probation evictions. On the LIRS trace, the keys that come back inside that window are mostly loop or scan members whose next reuse is far away. Promoting them pollutes protected, which is exactly the scan resistance v002 had gained.

## Takeaway

On workload-a, letting a ghost hit bypass probation in this tiny per-bucket SLRU does more harm than good. A ghost hit is weaker evidence than a resident second hit, but it was given the same reward, and with a fixed protected cap every such reward costs a proven item. If ghost history is tried again, it should affect probation instead of protected. For example, it could insert the key at probation MRU with an extra life, or adapt the probation/protected split ARC-style. Simply letting ghost keys jump the queue does not work. The per-op-time shift should be re-measured before anyone credits or blames this change for it.
