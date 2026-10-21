# Variant v002 Insight

## Helped

- hit-rate: The gain over v003 is within noise. A stash victim scan that prefers counter-0 slots near the LRU end, and ages every skipped slot by one, does little on top of v003's MRU re-entry. Demoted ex-residents already sit at the MRU end with their `PROMOTE_THRESHOLD - 1` pre-credit, so the LRU-end victim was usually a fresh one-touch key anyway. In that case the scan picks the same item the plain LRU choice would have picked.

## Hurt

## Takeaway

On this trace, position-based protection (v003's `DEMOTE_TO_STASH_POS = Back`) and frequency-based protection (this CLOCK-like scan) protect the same items, so stacking them adds almost nothing. With `PROMOTE_THRESHOLD = 2`, any stash item with a counter above 0 is one hit from promotion and leaves the stash soon after. That leaves the scan few protected slots to skip, and the decrement-on-skip aging removes even that protection after a single pass. If stash retention is to be pushed further, the lever is probably not victim choice inside a bucket. It is more likely stash capacity or admission (H01), or a higher promotion threshold that lets counters build up. Caveat: the overlay source could not be listed in this session (no directory or search tools), so this reading rests on the hypothesis's change description and the two fitness vectors, not on a direct code diff.
