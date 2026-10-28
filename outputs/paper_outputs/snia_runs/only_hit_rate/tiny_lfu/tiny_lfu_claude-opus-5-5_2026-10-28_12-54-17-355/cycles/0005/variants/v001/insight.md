# Variant v001 Insight

## Helped

- hit-rate: web_0 at the small size gained clearly, and it was the only cell to move meaningfully upward. A miss-then-`put` now adds one increment instead of two, so the sketch takes about half as many increments per cold insert. At this size the sketch is narrow (width = 4 × capacity, rounded up to a power of two), so fewer increments mean less collision inflation in the count-min estimate. Cold window candidates then look less like the probation victims they are compared against, and the strict `>` contest rejects them more often.

## Hurt

- hit-rate: hm_0 at the small size went down. This is the cell H01 predicted would gain, so the hypothesis is refuted there. The premise was wrong. The double count added the same +1 to every key that entered through a miss, including the probation victims. Before the fix, a once-touched candidate (2) against a victim hit once after admission (3) loses the contest. After the fix it is 1 against 2 and still loses. The "one-hit newcomer beats reused victim" ordering H01 targeted never happened. The real change is narrower. In the parent, each re-miss of a key that had been evicted counted twice while a hit counted once, so keys that kept coming back after eviction built up frequency faster. hm_0's recency-driven reuse seems to have benefited from that bias at the small size. Removing it takes away an accidental recency boost.
- hit-rate: hm_0 at the large size and web_0 at the large size barely moved. At this capacity the sketch is wide and few keys get re-missed, so neither the collision effect nor the miss-weighting effect matters much. Any small drift probably comes from the window/protected climbers taking a slightly different path, not from the sketch change itself.

## Takeaway

The double count mostly added the same offset to every key, so fixing it did not change admission decisions for one-hit keys, which is what H01 assumed it would do. Its real effects are second-order. It reduces collision noise in a small sketch, which helped web_0 at the small size. It also removes a hidden bias that weighted misses double relative to hits, which hm_0 at the small size had been relying on. The single-count accounting is the correct base to keep. If hm_0 at the small size needs a recency boost on re-entry, add it on purpose, for example a ghost/doorkeeper bonus for recently evicted keys, rather than through the double count.
