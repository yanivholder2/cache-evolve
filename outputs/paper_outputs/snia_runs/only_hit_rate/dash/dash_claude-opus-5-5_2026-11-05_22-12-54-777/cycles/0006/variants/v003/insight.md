# Variant v003 Insight

## Helped

- hit-rate (hm_0, both sizes): `demote_from_normal_bucket` now halves `lfu_counter` on the direct path and the cascade path into the stash. A demoted item that target eviction already judged cold therefore no longer carries its full normal-tier count into `lru_tail_lfu_victim`. Ghost-credited arrivals (`lfu_counter = 1`) and recently re-touched stash items can now win the 8-slot tail scan against it. A demoted item at counter 1 falls to 0, the same as a fresh key. This gives a small, consistent gain at both cache sizes. That fits hm_0, which has the reuse the ghost exploits, as cycle 0005 v001 showed.

## Hurt

- hit-rate (web_0, size 131072): A slight, noise-level dip. With a large stash, some demoted items do come back. They now return to target with half their earned count, so a few lose the target-tier LFU comparison that the parent's full counter used to win them.
- hit-rate (web_0, size 16384): Unchanged. This trace's scan keys rarely come back and the ghost rarely fires (as in the parent). So there are almost no ghost-credited arrivals for the lower demoted counters to give way to, and the stash victim choice is effectively the same.

## Takeaway

Ageing at the normal→stash boundary is safe and helps a little, but only where ghost re-admissions exist to take advantage of it. It moved hm_0, and web_0 is flat to slightly negative. This partly answers the "move stash ageing rather than remove it" question: adding ageing at a point where no hit is penalised costs nothing on hm_0, unlike removing stash-hit halving. The effect is small, though, because halving shrinks counters that the cap-15 halve-on-hit ageing already keeps low. Stale demoted frequency was a minor shield in the stash, not the main limit on hit-rate.
