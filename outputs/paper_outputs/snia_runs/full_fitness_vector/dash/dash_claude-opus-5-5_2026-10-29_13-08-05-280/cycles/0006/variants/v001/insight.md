# Variant v001 Insight

## Helped

- hit-rate: Three of four cells rose, including both 16384 cells, even though the stash-side size gate was removed. Removing the gate should have cost small caches a little. Gains here mean the direct swap more than made up for it. The swap keeps the demoted probing entry in the target slot it frees. The parent's cascade instead evicted a probing-LRU or target-LRU item, or pushed one into the stash. So a guarded promotion no longer throws a resident key away, and that key can still hit later. The hm_0 131072 cell dipped slightly. One possible cause: the demoted entry lands at the target's `Back` with `lfu_counter` reset to 0, so in a large cache it can push out a colder but still-useful target entry sooner than the parent's ordering did.

## Hurt

- per-op-time (get p50 and p99): The hypothesis aimed to lower get latency, but both percentiles got worse, the tail by a lot. The swap runs inline on the get path: it does two removes, one insert at `promote_pos`, and a write-back across two buckets on every guarded hit into a full probing bucket. The parent's cascade often ended early or was skipped. Because the swap never frees a slot, probing buckets stay full, so more guarded hits take this arm. The work isn't removed, just moved onto more reads.
- per-op-time (put p99): Put code was not edited, yet the put tail rose clearly. The likely cause is occupancy. The parent's cascade evicted items during gets, which left free slots and stash room for later inserts. The swap keeps the item count the same, so buckets stay saturated. More puts then hit the slow full-bucket path: displacement, stash, then eviction. Dropping the size gate may add a little to this in small caches, because entries now need a third hit to leave the stash.

## Takeaway

Swapping two buckets directly on a full-probing promotion is a hit-rate change, not a latency win. It keeps entries the old cascade would have evicted, which helps hit-rate. But the buckets then stay full, so the extra cost lands on the get and put tails. Cycle 0005 v002's speedup did not carry over once combined with the 24-byte, per-segment-tick parent. The hit-rate results also suggest the size gate was redundant: small-cache cells improved without it. Caveat: I wrote this from the hypothesis and both fitness vectors. I could not list the overlay source in this session, so the code-level reasons assume the hypothesis was implemented as written.
