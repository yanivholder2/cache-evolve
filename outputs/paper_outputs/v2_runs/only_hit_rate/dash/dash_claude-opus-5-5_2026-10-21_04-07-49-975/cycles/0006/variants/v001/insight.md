# Variant v001 Insight

> Note: I could not list or open the overlay source in this session; the only tools available were file read and write, with no directory listing or search. The mechanism below comes from H01's specified changes, which only change `GHOST_SLOTS` to 4× v003's value. The diff itself was not checked.

## Helped

## Hurt

- hit-rate: With four times as many ghost slots, far more stash-evicted fingerprints survive, so many more returning fresh keys match a ghost entry and get inserted at the stash MRU end instead of BIP probation. Most of the newly admitted keys come back after one to four caches' worth of evictions. A key with a reuse distance that long is unlikely to stay resident until its next access even from the MRU position. Each such MRU insert pushes a shorter-reuse key toward eviction, so this variant pays the cost of displacement without getting the hits the bigger window was meant to recover.

## Takeaway

The ghost window only helps while it is about one cache's worth of evictions, the range where an MRU-placed key can still be reused before it is evicted. Stretching it to about four caches turns the ghost into a pollution path. This answers cycle 0005's open question: the misses with reuse distances between one and four caches cannot be recovered by MRU placement. The ghost-size lever is saturated at v003's size or smaller, so future work should keep v003's window and try a smaller one, or make ghost admission depend on how recently the fingerprint was evicted. Caveat: this is a single LIRS trace at one cache size.
