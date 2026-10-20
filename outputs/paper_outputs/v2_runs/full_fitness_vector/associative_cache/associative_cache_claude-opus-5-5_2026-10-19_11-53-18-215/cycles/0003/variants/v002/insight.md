# Variant v002 Insight

## Helped

- per-op-time (get): get p50 and p99 both fell. Two changes did this. First, `find()` now uses `rposition`, so lookups start at the protected MRU tail and re-referenced hot keys match after a few compares instead of after walking past probation and most of protected. Second, a hit on the key already at protected MRU now skips the rotate. Hits that land near the tail also have a shorter `items[pos..].rotate_left(1)` span.
- per-op-time (put): put improved more than get, and put's p99 improved the most. The full-bucket miss path used to do `remove_at(0)` (shifting the whole `Vec` left) and then `insert(len0, ..)` (shifting protected right). It now does one `mem::replace` of `items[0]` plus `items[0..len0].rotate_left(1)`. That move covers only probation, about 1/4 of the bucket, and never touches protected. This was the parent's most expensive operation, so removing it explains why the put tail shrank so much.

## Hurt

(none) hit-rate and metadata-size did not move. Segment membership, promotion, demotion and victim choice are the same as in the parent, and `Item` and the bucket layout are unchanged.

## Takeaway

The parent's latency regression came from how data moved, not from the SLRU policy. Changing the scan direction and limiting the eviction move to probation recovered latency, and hit rate stayed identical to the parent's. Some cost is left. `Bucket::put` re-finds the new key with the now-reverse `find`, and the new item sits at probation MRU (`len0 - 1`), so that second scan walks all of protected first. A probation hit also still rotates across all of protected. A follow-up could return the insert index straight from `pair_put`, and could make promotion cheaper, for example by placing promoted items with a swap or ring offset.
