# Variant v003 Insight

## Helped

- hit-rate (web_0 @ 131072): This cell beats the parent, so the refutation clause did not trigger. Some fresh promotions still get into main after the correlated-reference filter, and the veteran grace still outranks them with blocks that were re-hit in main. The gain is less than half of what the same bit added on top of v002's own parent, though. So the two changes do partly overlap: many of the counter-0 promotions the grace used to outrank are burst-only blocks that the filter now keeps out of main.
- hit-rate (hm_0 @ 131072): This cell rose a little instead of dropping below the parent. The grace is one-shot and is earned only by main hits. A stale veteran therefore loses its extra step on the first visit where its counter is 0. It does not stay ahead of the filtered, real promotions for long.
- hit-rate (web_0 @ 16384, hm_0 @ 16384): Both cells were flat to slightly up. In 128-slot buckets the filter window is only 4 insertions, so many burst promotions still reach main there. The veteran bit gives a small extra benefit against them. That pushes web_0 just past the predicted flat band, consistent with the grace doing more where the filter does less.

## Hurt

None. No cell moved down.

## Takeaway

The veteran grace and the correlated-reference filter combine, but they are not independent. Both target counter-0 promotions, so once the filter is in place the grace gains only part of what it did on its own. It still costs nothing on the hm_0 cells. The bit is worth keeping in this lineage for one byte per slot. Most of its leftover value probably comes from the narrow-window 128-slot buckets. If the filter window gets wider there, as the parent's insight suggests, expect the grace's gain at 16384 to shrink further.
