# Variant v001 Insight

## Helped

- hit-rate: Both large-cache cells (131072) improved over the parent, with web_0 large gaining the most. In a long window, one early hit no longer wins the tie. The candidate also needs a hit in the last quarter of its own residency. Most long-resident candidates fail that test, so the rule falls back to strict `>` for them. Long-resident probation victims are no longer displaced by stale newcomers, which was the `>=`-like failure the parent's sticky `reused` flag caused.
- hit-rate: hm_0 large recovered slightly, for the same reason. Fewer candidates pass the tie gate when the window is large, so warm probation entries survive more often.

## Hurt

- hit-rate: web_0 small gave back part of the gain it made in the parent. With a short window, a candidate's residency, measured in inserts, is short. A useful hit that came soon after arrival can then still fall outside the last-quarter cutoff. The late-hit test throws out some of the short-reuse evidence that the boolean flag got right at 16384 entries.
- hit-rate: hm_0 small was essentially unchanged. The tie rule still has little to act on there, because window hits also bump the sketch, so reused candidates usually win under `>` anyway.

## Takeaway

Making the tie-break depend on when the window hit happened, relative to the candidate's own residency, mostly removed the large-cache losses the sticky flag caused. A fixed "last 1/4" cutoff is too strict for short windows, though. It swaps some of the small-cache gain for the large-cache recovery, and the summary moves only slightly. The useful signal sits between "any hit" and "late hit", so the next lever is the cutoff fraction (`LATE_HIT_DIVISOR`). It could also be allowed to scale with window size rather than stay a constant. hm_0 small is still not affected by changes to the tie rule.
