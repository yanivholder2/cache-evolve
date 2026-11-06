# Variant v002 Insight

## Helped

(none. hm_0 is unchanged at both sizes, so the predicted gain on hm_0 did not happen.)

## Hurt

- hit-rate: web_0 dropped by a rounding-level amount at both sizes, which is effectively neutral. So the claim that this "will not hurt web_0" holds. A tiny cost is consistent with a small number of demoted counter≥2 blocks taking a slot at the stash MRU end that a later one-touch block would otherwise have used.

## Takeaway

Gating the stash landing position on `lfu_counter >= 2` is almost entirely redundant with the parent's frequency-aware victim choice. `lru_tail_lfu_victim` already evicts the lowest-counter item among the 8 least-recent slots, with ties going to the older slot. A demoted block with a counter of 2 or more that lands at `Front` can only be evicted if every other item in that window has a counter at least as high. Moving it to `Back` changes nothing whenever one-touch items are nearby, and it adds little protection when the window holds a block's whole stash bucket. So the code change was implemented exactly as specified (the diff is confined to `stash_demotion_pos` and its two call sites), but the hit-rate barely moved. Once the victim choice reads the counter, recency position stops being a lever for blocks that already have high counters. If hm_0's long-gap reuse is to be served, it needs either more stash capacity or a change to *which* items the tail scan considers, such as the window width or how counters age, not a change to where demoted items are inserted.
