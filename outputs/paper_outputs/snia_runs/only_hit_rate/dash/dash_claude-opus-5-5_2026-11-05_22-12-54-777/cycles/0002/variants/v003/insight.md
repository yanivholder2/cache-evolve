# Variant v003 Insight

## Helped

- hit-rate (web_0, size 131072 only): The larger normal tier (28 normal / 4 stash) gave a marginal gain on this one trace/size. Here the reused set still fit and was still promoted, so the extra protected space kept a few more blocks. The gain is too small to call real, so the 3/4 split was already close to this trace's ceiling.

## Hurt

- hit-rate (hm_0, size 131072): This trace/size had a large regression. That one result drives the drop in the summary. Every new item is admitted into stash, so halving the stash from 8 to 4 buckets halves how long a block can wait for its second touch before it is evicted. hm_0's reused blocks appear to come back after longer gaps than a 4-bucket stash can hold. They are evicted before they can be promoted, so the larger normal tier is never filled with the right blocks.
- hit-rate (both traces, size 16384): Both had small losses. At the tight size the shorter stash wait costs more than the extra normal capacity gains, which matches the hypothesis's "neutral or slightly worse" prediction.

## Takeaway

Under "admit everything to stash," the stash is the only path into the protected tier as well as a scan filter. Shrinking it to 1/8 starves promotion before it hurts scan filtering. So the risk the hypothesis expected on scan-heavy web_0 showed up on reuse-heavy hm_0. The v002 trend (larger normal tier is better) stops between 3/4 and 7/8. The 24/8 split stays the better topology, and further gains likely need promotion or admission changes, such as giving stash entries a longer wait or promoting on first re-reference, rather than more ratio sweeps. (Note: I could not inspect the overlay source for this write-up. I assume v003 changed only the bucket constants, as the hypothesis specified.)
