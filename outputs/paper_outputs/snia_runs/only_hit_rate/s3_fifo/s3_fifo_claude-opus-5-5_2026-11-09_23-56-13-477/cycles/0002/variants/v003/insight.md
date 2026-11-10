# Variant v003 Insight

## Helped

- hit-rate (hm_0, 16384): moving 10% of capacity from small to main gives hm_0's working set more room under second-chance protection, and the ghost queue (bounded by `main_size`) grows with it. Keys pushed out of the shorter small window are more likely to still be in ghost when they come back, so they go straight into main.
- hit-rate (summary): the hm_0 small-cache gain is slightly bigger than the losses elsewhere, so the aggregate inches up. The difference is close to noise, not a clear improvement over v001.

## Hurt

- hit-rate (web_0, 16384): with only 15% of capacity in small, a web_0 key now has to come back within a shorter window to graduate on a single reuse. Keys whose reuse gap falls between the 15% and 25% windows go to ghost and pay one extra miss. A smaller small queue lets fewer one-reuse keys through, but that didn't reduce main pollution enough to make up for it.
- hit-rate (hm_0, 131072): this moved by a tiny amount. At the large size, the small queue is big enough either way, so the split hardly matters. The movement is in the opposite direction from 16384, so the split doesn't have a consistent effect at this size.

## Takeaway

Shrinking small from 25% to 15% on top of v001's single-reuse graduation barely changes anything, and the effect flips sign between traces and cache sizes. That answers H03's question: a static split between 10% and 25% doesn't recover v001's web_0 loss, because the loss comes from the one-reuse admission rule letting "accessed exactly twice" keys into main, not from how big small is. The split is a weak lever here. Further gains should come from filtering what gets into main, for example a reuse-rate-aware or occupancy-aware threshold (H02's adaptive direction), not from tuning the static split. Only hit-rate was measured. Metadata-size should be unchanged, since only the queue sizes changed and the `Entry` layout didn't.
