# Variant v001 Insight

## Helped

- hit-rate (hm_0, size 16384): This is the only cell that moved by more than noise, and the gain is small. It is consistent with the hypothesis's mechanism. With `GHOST_ADMIT_COUNTER = 2`, a ghost-credited key now beats single-hit (counter 1) residents in `lru_tail_lfu_victim`, not only fresh (counter 0) arrivals. It also keeps a non-zero counter after one halve-on-cap pass (2/2 = 1 rather than 1/2 = 0). This size is where the parent's cache-sized ghost fires most often on hm_0, so this is also where a stronger credit has the most keys to act on.

## Hurt

- hit-rate (web_0, size 16384): The dip is at noise level. Over-crediting scan keys could in principle push out real single-hit residents, but web_0's ghost seldom fires (per the parent insight), so the bigger bonus rarely gets applied there. Keeping 16-bit fingerprints avoided the false-positive ghost hits that hurt web_0 in cycle 0005 v002.
- hit-rate (hm_0 and web_0, size 131072): Effectively flat. Doubling the bonus did not recover the large-size loss on hm_0. As the hypothesis's rationale anticipated, that rules out admission strength (at least up to +2) as the binding limit there.

## Takeaway

On its own, with full fingerprints, a stronger ghost-admission credit is safe but barely matters. The extra step only helps against stash peers that already have exactly one hit, and in a stash made mostly of counter-0 arrivals, a credit of 1 already wins the 8-slot tail scan. The unchanged hm_0 result at 131072 settles the parent's open question: the large-size loss is not about ghost reach or bonus weight. The next change should target something else, such as where ghost-credited keys are placed, or the tail window and ageing in the normal tier.
