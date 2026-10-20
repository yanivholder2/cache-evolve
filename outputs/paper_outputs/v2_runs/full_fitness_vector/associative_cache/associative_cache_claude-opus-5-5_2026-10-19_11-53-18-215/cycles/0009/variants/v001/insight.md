# Variant v001 Insight

## Helped

- hit-rate: The new `protected_cap` widens protected only in buckets of 32+ slots: 32 -> 30 instead of 28, and 64 -> 60 instead of 56. In those buckets probation shrinks to 2 or 4 slots, but it still has the LRU and MRU positions that v003's cold-entry rule needs. So the cap gain and the LRU-admission gain add up rather than cancel. The gain is small. That fits a change that only reclaims a few more slots for re-referenced items, and only in the larger buckets. At 16 slots the new formula gives 14, the same as v003's, so those buckets behave exactly as before.

## Hurt

- per-op-time (get): The median is flat, but the tail rose. A larger protected segment means a probation hit, or a hit deep in protected, rotates a longer `items[pos..]` / `tags[pos..]` span to reach protected MRU. More resident items also sit in protected, where every hit pays that rotate.
- per-op-time (put): The tail rose here too. With a narrower probation, keys are evicted sooner and come back more often as ghost hits. A ghost hit takes the expensive path: `evict_front` (a `remove(0)` shift) plus `insert(len0)` on both `items` and `tags`. Resident-key puts also hit the longer rotates described under get. This is the opposite of the in-place `mem::replace` saving that v003 bought.

metadata-size did not move: `Item` and the cache header are untouched.

## Takeaway

The floor-of-2 clause never actually takes effect where it was meant to. At 16 to 31 slots v003 already left probation at 2 or more slots, so this variant is really "the 15/16 cap applied to buckets of 32+ slots." Its result shows that cap and LRU cold entry are complements, not substitutes. They stack only barely, though, and the extra hit rate costs tail latency on both get and put. That suggests the cap axis is close to saturated: squeezing probation further now mostly buys longer shifts and rotates.
