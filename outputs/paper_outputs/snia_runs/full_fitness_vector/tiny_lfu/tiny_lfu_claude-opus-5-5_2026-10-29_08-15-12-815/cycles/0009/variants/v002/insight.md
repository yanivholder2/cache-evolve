# Variant v002 Insight

## Helped

- per-op-time (get/put p50 and p99): Every percentile is faster, but the change can't explain it. `get` is byte-for-byte identical to the parent. On the `put` side, the miss→put path now does *more* work: `tick()` adds an increment, a compare, and sometimes a `reset()`. This is the second large latency swing in a row on code that did not change on the hot path. It confirms that per-op-time differences of this size are benchmark noise or layout effects, not signal.

## Hurt

- hit-rate (hm_0 @ 16384, web_0 @ 16384): Both small-size cells gave back the parent's gains. The miss→put `tick()` counts every miss→put pair as two additions instead of one, so the sketch halves more often than under the parent's dedup. At 16384 the aging period is short (163840 additions), so frequency history now decays before moderately popular keys can build estimates above the probation victims. This is the "drops back the way v001 did" outcome named in H02, and it shows the small-size gain came from slower aging, not from the tie-break.
- hit-rate (web_0 @ 131072): This was the target cell, and it did not improve. The parent's guarded tie-break had already recovered the large-size newcomer edge. Restoring the clock on top of that adds no value at this size, so the clock fix and the tie-break were overlapping fixes for the same loss, not independent ones.
- hit-rate (summary): Three cells slipped and none gained, so the combination is a net loss against the parent.

## Takeaway

The faster aging clock and the `TIE_WIN_MIN_FREQ = 2` tie-break do not stack. With the tie-break in place, counting deduplicated miss→puts toward `additions` gains nothing at 131072 and costs hit-rate at 16384. The parent's slower clock (dedup with no tick) was a real benefit at small sizes, not a side effect to undo. The next step is to keep the parent's no-tick dedup, or to try a size-dependent `SAMPLE_FACTOR` that ages more slowly at small capacities. Re-adding a uniform tick is not the way forward.
