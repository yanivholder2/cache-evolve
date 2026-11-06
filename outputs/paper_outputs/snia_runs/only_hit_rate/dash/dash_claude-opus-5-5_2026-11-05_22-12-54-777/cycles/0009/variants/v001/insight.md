# Variant v001 Insight

## Helped

- hit-rate (web_0, size 131072): The predicted gain arrived, and this cell won back part of what v002's strict gate cost it. Under the relaxed gate, the bypass now proceeds when the coldest tail-window slot has counter 1. That covers the "all-warm" windows the parent saw, which turn out to be mostly counter-1 slots, not multi-hit ones. So returners that would have gone to the stash and been dropped before reuse now land in the target bucket.

## Hurt

- hit-rate (hm_0, size 16384): This was the biggest move in the variant, and it went the wrong way. It is the exact case the hypothesis named as its falsifier. In contended small targets on hm_0, counter-1 residents (earlier ghost arrivals or fresh Front arrivals that aged into the tail) are often reused again. Admitting a returner over them gives back much of the protection v002's counter-0 gate provided.
- hit-rate (web_0, size 16384): A slight drop near noise level, in the same direction as hm_0. Some small-size swaps of a counter-1 resident for a one-time returner have come back.

## Takeaway

Moving the bypass threshold from counter 0 to counter ≤ 1 shifts the trade-off between cache sizes instead of improving both. It frees useful returners on web_0 at 131072. But at 16384, counter-1 residents are worth protecting, especially on hm_0, and the loss there outweighs the large-size gain in the mean. One global threshold on the window minimum looks exhausted. A size- or contention-dependent threshold looks like the next step: ≤ 1 for large or uncontended targets, == 0 for small or contended ones. Another option is a tie-break that prefers the resident at equal counter. hm_0 at 131072 stayed flat, consistent with its tail windows rarely bottoming out at counter 1.
