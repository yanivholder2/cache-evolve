# Variant v002 Insight

## Helped

_(No dimension improved. The ghost history was kept off `Item` as designed — `ghost: Vec<K>` and `reuse: Vec<bool>` both live in the unmeasured bucket struct — so metadata-size held at the 16-byte floor, but that is a hold, not a gain.)_

## Hurt

- hit-rate: The 2Q ghost bet backfired — admitting every re-referenced (recently-evicted) key with `reuse = true` over-protects the bucket. Returning one-hit-wonders that arrive under a scan burst now enter already-shielded, so `eviction_candidate_index` finds fewer `false` bits, falls back to the LRU front more often, and evicts genuinely hot residents instead, dropping hit-rate below the parent's reuse-bit ceiling.
- per-op-time: Every `put` now pays the ghost bookkeeping — `take_ghost` linear-scans the ring on each miss, and `record_ghost` on eviction does a dedup scan plus an `O(bucket_size)` `remove(0)` front-shift — pushing put p50 up ~7x and lifting both put and get p99 tails over the parent.

## Takeaway

Giving the bucket memory across eviction does not lift hit-rate here: admitting ghost-hit keys as protected is too coarse, because a re-reference is not proof of long-term heat, and blanket protection erodes the reuse bit's ability to single out one-hit-wonders — the very discrimination that earned the parent 0.7267. The off-`Item` ghost ring preserved the 16-byte floor, but its per-op cost and its inflationary effect on the protected set both cut the wrong way.
