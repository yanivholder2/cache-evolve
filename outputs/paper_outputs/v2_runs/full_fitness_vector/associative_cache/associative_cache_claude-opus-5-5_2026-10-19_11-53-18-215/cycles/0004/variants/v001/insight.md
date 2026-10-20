# Variant v001 Insight

## Helped

- per-op-time (put): put got faster at p50 and p99. Two parts of the change account for this:
  - **Probation-only eviction.** The parent's full-bucket miss did `remove_at(0)` and then `insert(len0, ..)`. That shifted every protected slot twice. The new code does `mem::replace(items[0])` and then `rotate_left(1)` over `items[0..len0]`, so only the short probation segment moves (at most 1/4 of the slots).
  - **No re-scan.** `put_indexed` returns the final index, so `Bucket::put` no longer re-scans the bucket after inserting. That re-scan used to walk through all of protected to reach the new item at probation MRU.
- per-op-time (get p99): the tail improved slightly. This fits skipping `rotate_left(1)` on a protected hit that is already at MRU, and finding hot protected-tail keys sooner with `rposition`.

## Hurt

- per-op-time (get p50): the median get got slower, and get p50 is the dimension's summary, so the summary regressed and the hypothesis's "below 55" target was missed. With MRU-first `rposition`, a hit on a probation key, such as the second touch of a recently inserted key that is about to be promoted, now has to scan past the whole protected segment first. Misses still scan every slot either way. The new `pos != last` branch also sits on every protected hit. With 24-byte slots the scan was already cheap, so neither effect gets hidden.

## Takeaway

hit-rate and metadata-size did not change, which confirms the four edits kept SLRU semantics and the 24-byte `Item` exactly. The movement fixes split by operation:
- **The put-side fixes stack cleanly with narrow slots.** These are probation-only eviction and returning the index instead of re-finding. They remove whole-segment memmoves and a redundant scan.
- **MRU-first lookup is not a free win on this layout.** In SLRU, probation sits at the low indices, so `rposition` makes probation hits pay for scanning protected first. Compared with v001's forward scan, that cost the typical get more than the skipped no-op rotate saved.

The next step is to keep the put changes and revert `find` to `position`. Alternatively, test whether a segment-aware lookup helps: scan probation MRU→LRU first, then protected.
