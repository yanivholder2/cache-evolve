# Variant v001 Insight

## Helped

- hit-rate (hm_0, both sizes): Inserting promoted items at the MRU end of the destination bucket, on top of v002's 24/8 topology, lifted hm_0 at both sizes. The gain at 16384 was much larger than at 131072. This is the small-cache improvement that v002's insight said topology alone could not deliver. hm_0's reused blocks now survive long enough in the target tier to be hit again, instead of landing near the eviction end straight after promotion.
- hit-rate (summary): The hm_0 gains outweigh the web_0 losses, so the aggregate rose over the parent.

## Hurt

- hit-rate (web_0, both sizes): web_0 dropped slightly at 16384 and at 131072. So the hypothesis's claim of "no web_0 regression at 131072" did not hold, though the regression is smaller than the hm_0 gains. A likely cause is that the stricter probing trigger (MRU position plus a prior hit in the current target bucket) and the mid-stash demotion slot keep web_0's weakly-reused blocks resident longer. Those blocks then take space from blocks that would have produced hits.

## Takeaway

On v002's 24/8 split, MRU-end promotion is what finally moves the small-cache case, but only on hm_0. web_0 gave back a little at both sizes, so the guards against v001's web_0 regression reduced it without removing it. All three changes landed together, and the overlay source was not inspected for this insight, so this run cannot say which piece causes the web_0 loss. A child should isolate them: keep MRU promotion, then toggle mid-stash demotion (most likely to keep stale demoted items too long on web_0) and the hit-bit probing guard one at a time.
