# Variant v002 Insight

## Helped

- per-op-time (`get` p50): The median `get` got faster while hit rate stayed bit-identical. The likely reason is that a normal-tier hit now does one AND against `stash_masks[stash_idx]` instead of running a linear `get_position` key-compare scan over the stash bucket. This supports the parent insight's claim that the stash probe on the common hit path is a real part of the p50 gap.

## Hurt

- per-op-time (`get` p99): Tail gets got much slower. Two causes are likely. First, gets that hit in stash or fall through a false-positive bit still pay the full stash scan plus the mask test. Second, a stash hit that promotes the item out of stash now also pays for `refresh_stash_mask`, which re-hashes every remaining stash item. That recompute lands on exactly the slow path that was already the tail.
- per-op-time (`put` p50, p99): Both put percentiles regressed sharply. The median moved too, so the mask upkeep is not rare. Under a full cache, most puts go through an eviction or demotion cascade that removes an item from a stash bucket. Each such removal now triggers a full `refresh_stash_mask` re-hash. The separate `stash_masks` Vec also adds a second memory touch, away from the bucket data, on every stash insertion.

## Takeaway

The stash-presence mask does pay off on the read median, which confirms the stash probe as a p50 cost. But recomputing the mask on every stash removal pushes that cost onto `put` and the `get` tail, and the tradeoff is clearly bad there, because in this design stash removals are on the common put/eviction path, not rare. A follow-up should keep the AND-test fast path while making removals O(1). Two options: a small per-bit counting mask, or storing the mask bits next to the stash bucket's metadata so no re-hash or extra cache line is needed.

(Note: I could not list or open the overlay source in this session, so this analysis rests on the hypothesis's specified changes and the parent/child fitness vectors, not on a line-by-line diff.)
