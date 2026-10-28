# Variant v003 Insight

## Helped

- hit-rate (16K, `web_0` and `hm_0`): Both cells, the ones the hypothesis aimed at, moved up a little. Refusing a gated stash→full-target promotion when the promotee's sketch estimate is below that of the target's LRU resident (index 0) stops a single burst of two stash hits from pushing out a resident with a longer access history. The refused item stays at the stash MRU end with its counter at `STASH_PROMOTION_HITS - 1`, so it can still promote on its next hit once its estimate catches up. Very little of the hoped-for gain at the small size appeared, though.
- hit-rate (128K, `web_0` and `hm_0`): These cells gained at least as much as the 16K cells, maybe a little more. The aging window scales with segment slots (`SKETCH_SAMPLE_FACTOR × total slots`), so the larger cache has a longer frequency memory. Its sketch can tell residents with real history from one-off stash visitors, while at 16K frequent halving keeps both estimates small and close. Ties admit, so the check usually lets the promotion through when the estimates are close. This is the likely reason the small cells gained less.

## Hurt

None of the cells regressed. Refusal only blocks a promotion that would have displaced a resident, and the refused item keeps its stash slot and its near-gate credit. That makes the change one-directional: at worst it postpones a correct promotion by one hit.

## Takeaway

TinyLFU-style admission works once frequency is stored outside the item, survives moves and evictions, and decays by halving. Unlike c2's in-item `lfu_counter` comparison, this produced a small gain in every cell with no regressions, and it did so by making promotion stricter, not easier. The gain did not concentrate at 16K, so `web_0` 16K is still not unlocked by checking who gets displaced. The likely limit is that a short, frequently halved sketch at small sizes cannot separate the candidates well. Before applying the sketch anywhere else, test whether a longer or fixed aging window fixes that.
