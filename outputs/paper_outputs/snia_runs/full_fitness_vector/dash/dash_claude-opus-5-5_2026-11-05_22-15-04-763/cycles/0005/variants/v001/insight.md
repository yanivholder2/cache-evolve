# Variant v001 Insight

## Helped

- hit-rate: Not a gain, but a passed control. Every trace and size matches v002 exactly. This confirms that switching `get_position` from `position` to `rposition` returns the same index (keys are unique within a bucket), so placement and eviction did not change at all.
- per-op-time (`put` p50): The median put improved a little. This fits the reverse scan in `put_at`'s update-in-place check, which now finds a re-written hot key at the MRU end sooner. The shift is small, though, and most puts insert absent keys, which still need a full scan. Treat it as weak evidence, not a proven win.

## Hurt

- per-op-time (`get` p50/p99): The main claim fails. `get` p50 did not drop and moved slightly the wrong way, and p99 also ticked up. A `get` probes several buckets: stash, then target, then probing. Every probed bucket that misses still has to scan the full bucket, whichever direction it runs. Only the one bucket that hits gets the shorter scan. With small buckets, the comparisons saved there are too few to show up next to the misses and the move-to-MRU shift in `get_from_position`.
- per-op-time (`put` p99): The put tail got worse even though the scan direction cannot add work to a put. The most likely cause is the tail cost v002 already showed: the "both full → target + demotion cascade" fallback, which this change did not touch, plus run-to-run noise. The reverse scan is probably not the cause.

## Takeaway

Scan direction inside a bucket has been ruled out as the cause of the `get` p50 gap. Reversing the scan gives exactly the same hit rate and does not move the median `get`, because per-bucket comparison count is not the bottleneck: misses in the stash and probing buckets still need full scans. Following H01's own fallback reasoning, the next target should be how many buckets each hit has to probe, such as the order of the stash check or skipping it, or the `Vec::remove`/`push` shift cost of move-to-MRU. Shortening the key compare loop will not close the gap.
