## Recurring wins

- Changes that only affect efficiency or layout and leave behaviour identical are the most reliable gains. Cheap hashing (c1), compact `u32` links (c2), CLOCK ref bits in place of LRU links (c6), and packing flags into link bits (c5) all cut cost or metadata and left hit rate unchanged.
- Every hit-rate gain so far has come from making the sketch more accurate or the duel's inputs fairer: conservative update (c3), `u16` counters at twice the width (c4), and counting each access once (c5). Each gain was small, and each was smaller than the one before.
- The fused, register-held sketch increment is the only latency change with a mechanism in the code that recurs. Its gain showed up in c4 and c6. It was absent in c5, which was most likely bench drift.
- Variants that change one thing at a time on the latest base produce attributions you can read. Merges of orthogonal changes have stayed bit-for-bit identical on hit rate.

## Recurring failures

- Any recency-oriented structure loses on workload-a, which rewards stable frequency filtering at the door:
  - W-TinyLFU window + SLRU (c1)
  - a ~1% window (c5)
  - a 10% window (c6)
- Sketch aging or halving lowered hit rate every time it was tried (c2 with 4-bit counters, c4 with full-width counters plus conservative update). Decay removes the margin that keeps scans out.
- Tweaks to the admission rule do nothing or hurt. Letting newcomers win ties (c3) hurt, and frequency-aware victim sampling near the LRU end (c2) did nothing. The strict `>` duel is not over-rejecting.
- Sketch layouts that break row independence, such as a blocked or cache-line layout (c3), add correlated collisions and inflate estimates. The sketch already fits in cache.
- Extra work on hot or rare paths costs latency right away: per-op branches, extra hashing on a miss, unbounded CLOCK sweeps, and re-hashing during backward-shift deletion. Tail regressions come from these rare paths.
- A custom index table that stores only node indices (c6) does worse than SwissTable, because each probe has to load from `nodes`.
- Per-op timing is noisy across cycles. Several variants sped up or slowed down with no cause in the code. Treat a latency claim as unverified until it has been measured side by side against the parent.

## Open questions

- Can word-level bit scanning (`trailing_ones` over `u64` ref bits) bound CLOCK's sweep and bring `put` p99 back to LRU's level?
- Does CLOCK combined with the fused increment keep both gains when measured side by side against each parent?
- If relinking and duplicate hashing don't set the median `get` cost, what does? Map probing, key hashing, or memory layout of the nodes?
- Would a map that stores hash tags or home slots beat std `HashMap`, or is the map not a bottleneck at all?
- Is workload-a's hit rate at its ceiling for the TinyLFU family? If it is, the remaining work is metadata and tail-latency reduction only.
- The unkeyed fast hasher has never been checked against adversarial or collision-prone keys.

## Per-policy notes

- **Baseline TinyLFU (strict-`>` duel against LRU head, no window):** this remains the best-performing policy structure. Its hit rate is set by how accurate the estimates are, not by tie rules or exact recency order.
- **CountMin sketch:** conservative update plus `u16`/2× width is close to its accuracy limit. Saturating `u16` counters are enough. Further work on width, accuracy or micro-optimisation is used up, apart from the fused increment.
- **Aging/decay:** this has failed with both counter resolutions and both update rules. Drop it for this workload. At most, a very mild form, such as a much longer period or counting only misses, is untested.
- **W-TinyLFU / admission window:** this has failed at 1% and at 10%, with and without SLRU. Drop it.
- **LRU vs CLOCK:** CLOCK keeps hit rate the same and shrinks per-entry metadata. It is the current base, but its eviction sweep has to be bounded.
- **Victim selection:** sampling near the LRU end gives no information, because those entries' estimates are nearly equal.
