# Variant v002 Insight

## Helped

(none) No dimension improved. The `metadata-size` summary tracks only `Node<u64,u64>`, which stays at 16 bytes because the sketch lives outside the node. The flat summary therefore says nothing about the new memory cost (see Hurt).

## Hurt

- hit-rate: The duel did not lift hit-rate above the S3-FIFO plateau. It fell slightly below the parent, for three likely reasons:
  - **Every unvisited small-head exit now runs `find_victim()`, even when `x` loses.** The SIEVE hand sweeps further and clears main's VISITED bits much more often than real main evictions would. This weakens the second-chance protection of keys already in main.
  - **The sketch undercounts hot main keys.** It counts a hit only on a 0→1 VISITED transition, so a hot main key registers about once per hand pass. A candidate that keeps missing gets counted on every miss, which tilts the duel toward churny probationary keys.
  - **Each winning duel adds another fingerprint to the direct-mapped ghost.** This overwrites entries that the parent's ghost path already used to readmit returning keys.
- metadata-size: The `LruCache<u64,u64>` header grew because of the new `sketch` Vec, `sketch_mask`, `sample` and `sample_limit` fields. The roughly 4 KiB of count-min counters adds more heap memory that the 16-byte node summary does not capture.
- per-op-time: The `put` p50 and p99 rose because each miss now does a 4-row sketch increment. Each unvisited small-head eviction also does an extra hand walk, a second `hash_one` for the main victim and two 4-row `freq_est` reads. Occasional full-sketch aging passes over all 512 words add to the `put` tail. The `get` p99 also rose, because a key's first hit after its VISITED bit is cleared now pays for a re-hash and a sketch update inside `set_visited`.

## Takeaway

Adding a TinyLFU duel at small-queue exit on top of a ghost-equipped S3-FIFO/SIEVE cost latency and memory and slightly reduced hit-rate on workload-a. The ghost already readmits most keys that return, so the sketch mainly adds noise. Calling `find_victim()` speculatively on every probationary eviction also wears down SIEVE's visited bits. If frequency admission is tried again, it should:
- peek at the victim without clearing VISITED bits, and
- count main hits more faithfully than once per visited cycle.

Otherwise it is better to treat this as more evidence that the plateau is a property of the trace, not of the admission policy.
