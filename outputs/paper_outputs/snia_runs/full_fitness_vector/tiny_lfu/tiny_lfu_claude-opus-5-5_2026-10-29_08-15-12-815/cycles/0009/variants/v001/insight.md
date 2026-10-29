# Variant v001 Insight

## Helped

- hit-rate (hm_0 @ 131072): The one cell that improved. At the large size the window is 1% of capacity (1310 entries), so a candidate has a long stay in the window. Reaching an estimate of 3 there means it really was referenced again, and letting it beat a probation head with the same estimate replaces a stale incumbent with a newcomer that is just as hot and more recent. web_0 @ 131072 stayed flat, so the gain at this size is small and depends on the trace.

## Hurt

- hit-rate (web_0 @ 16384, hm_0 @ 16384, summary): Both small-size cells dropped clearly, and they pull the summary below the parent. That falsifies "raise hit-rate in all four cells". At 16384 the sketch is narrower, so counters collide more, and it halves after fewer additions, so estimates bunch up at low values. Ties at 3 are therefore common, and `admit` keeps handing them to the window candidate. Probation heads that had real but modest reuse get swapped out for newcomers with the same estimate. The double count already gives every newcomer a "seen again" bonus. The tie-break adds a second bonus pointing the same way, so the two mechanisms over-favour recency at this size instead of adding up.
- per-op-time (put p99): The put tail grew, and p50 did not move. Every tie the candidate now wins takes the three-list-op path in `evict_one` (unlink victim, unlink candidate, push to probation) instead of the single unlink on the loss path, so the costlier path runs more often on a full cache. That fits a cost in the tail only. The get p99 rise cannot come from this diff (`get` is byte-identical and `admit` is branchless), so treat it as benchmark noise.
- metadata-size: No change, as expected. The diff adds only a constant and an inline function.

## Takeaway

Matching the threshold to the double count (3 here vs 2 on the dedup line) puts the cut-off at the same number of real references, but that did not make the tie-break safe on the incumbent line. The miss→put double count and the explicit tie-break are not independent. Both favour the window candidate, and stacking them costs hit-rate at the small size, where low, colliding estimates make ties frequent. If the tie-break is kept on this line, it needs to scale with capacity: switch it off, or raise the threshold, where the sketch is narrow and the aging period is short. Otherwise drop it here in favour of the dedup line.
