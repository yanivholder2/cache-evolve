# Variant v003 Insight

## Helped

None of the dimensions improved in a meaningful way. `metadata-size` stayed the same as intended: the `Block([u16; 32])` table still reports `SKETCH_DEPTH * width * 2` bytes. The `get` p99 tail is flat, which fits with each access now touching one cache line instead of four, but the change is too small to count as a win.

## Hurt

- hit-rate: The drop is beyond the hypothesis's own refutation threshold, so this blocked layout fails the same way cycle 0003's did, even with disjoint lanes. The per-row collision rate is the same as the parent's, but two keys now share a block 8× more often than they share a row counter in the parent. When they do, each row has only 8 slots to choose from, so the chance that a cold key matches a hot key on all four rows is far higher than with the parent's independent double-hashed rows. Conservative update can't fix this: the hot key's counters are its own minimum and get bumped legitimately, so they inflate the cold key's min-estimate and skew the admission duel.
- per-op-time: `get` p50 went up, the opposite of the prediction. `indices()` adds an extra `fmix64` (two dependent multiplies plus shifts) to every sketch access, on the critical path before any load. The parent's 4-line access pattern was not actually stalling on memory: a sketch of a few tens of KB stays hot in L1/L2 under this benchmark, so saving cache lines bought nothing to offset the added ALU latency. `put` p50 and p99 also crept up, because each `put` runs the re-mix twice (once for `increment_hash` and once for the victim's `estimate_hash`).

## Takeaway

Putting a key's four counters in one block cuts sketch accuracy more than the per-row collision math suggests. It is the collisions across rows within a block that hurt estimate quality, and conservative update doesn't cover them. At this sketch size the counter table is cache-resident, so the memory-traffic saving doesn't exist, and the extra hash re-mix makes `get` slower. Blocked/one-line Count-Min layouts aren't worth pursuing here unless the sketch becomes much larger than L2. The remaining `get` latency more likely lies in the `HashMap` lookup or the hashing itself.
