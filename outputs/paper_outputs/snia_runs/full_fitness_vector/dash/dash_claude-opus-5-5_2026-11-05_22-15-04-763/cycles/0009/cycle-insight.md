# Cycle 0009 Insight

## Cohort outcome

All three variants built on 0008 v003 (the fast hasher plus the dense stash mask), and all three are Winners. v001 added a collision word so most stash-mask clears run in O(1). v002 switched the mask to a k=2 Bloom test in the same u64 word. v003 changed stash victim selection to spare a once-hit LRU item across a two-slot window. The two mask variants improved per-op-time with bit-identical hit rate. v003 is safe, but its effect sits inside noise.

## Patterns

- Cycle 0008's diagnosis held up. If the dense read-side mask word stays untouched, O(1) upkeep recovers `put` without the `get` p50 loss that 0008 v001/v002 suffered. In v001 the read structure was the only difference from those variants, and it was the thing that mattered.
- Within the one-word budget, the false-positive rate is now the main tuning axis for the filter. v002 sped up both `get` and `put` at no metadata or hit-rate cost, which suggests `put`'s pre-insert lookup goes through the same reject test.
- v001 and v002 change different parts of the mask: v001 changes how bits are cleared, v002 changes how keys map to bits. They look stackable, but v001's collision tracking assumes one bit per key. Under k=2, collisions happen more often, so more removals would take the fallback rebuild.
- The `get` tail still sits on the stash-hit promotion path: the item shift, plus the refresh fallback in v001. None of the mask changes fixed it, and v001 got slightly worse there.
- Stash victim choice based on per-item hit state is cheap and harmless. With a two-slot window, though, it almost never fires, so the hit-rate plateau has not moved. v003's timing gains on a change that only adds work are noise. All three variants have one sample each, so tail moves stay unconfirmed.

## Open questions

- Does v001's O(1) clear still pay off when combined with v002's k=2 mapping? It would need per-bit collision tracking so that either bit of a key can trigger the fallback. Is k=3 worth trying, given that fuller buckets shrink the benefit?
- How much of the `get` p99 is the promotion item shift, and how much is the refresh fallback?
- Does widening the stash victim window (evict the LRU-most never-hit slot, protecting each item at most once) move hit rate at all, or is stash eviction simply a weak lever on these traces?

## Continuation

Merge v002's k=2 mask with v001's collision-word O(1) clear on top of 0008 v003. Check that the fallback rate stays low under k=2, and try k=3 as a separate variant. For the `get` tail, go after the cost of the promotion shift directly. Give stash victim selection one more try with a whole-bucket window and a protect-once bound, and drop it if hit rate stays flat. Rerun any result that rests on a small tail change.
