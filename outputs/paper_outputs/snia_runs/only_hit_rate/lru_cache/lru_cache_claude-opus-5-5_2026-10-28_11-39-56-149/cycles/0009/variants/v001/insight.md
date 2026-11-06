# Variant v001 Insight

## Helped

- hit-rate (web_0 @ 16384, hm_0 @ 16384): Nothing moved, which is correct. `prot_adapt` is false at capacities ≤ 32_768, so `prot_pct` stays at 80 and the small-size path matches the parent exactly. This confirms the size gate isolates the change to the large size.

## Hurt

- hit-rate (web_0 @ 131072): This was the cell the climber was meant to help. Its first move (`prot_step = -2`, applied without a check because `prot_prev_hr` starts at 0.0) does lengthen probation. After that, the ±2 step never decays and flips whenever one 4-round aggregate is lower than the one before. Those aggregates change with web_0's phases far more than with a 2-point change in the split. So `prot_pct` drifts around instead of settling below 75, and it spends time at splits worse than the parent's fixed one.
- hit-rate (hm_0 @ 131072): A small loss. The climber has no stopping point and no decay, so it keeps moving `prot_pct` away from any good value. Each shrink demotes protected LRUs to probation right away, where hm_0's scan traffic can evict hot entries. A grow is filled lazily, so the aggregate measured right after a grow doesn't yet show the larger segment. The climber therefore sees growing at a disadvantage, which biases it toward the smaller protected share that hm_0 dislikes.
- hit-rate (summary): Both large-size cells regressed and the small cells stayed flat, so the summary fell below the parent's. The hypothesis expected each trace to push `prot_pct` toward its own best split, and that did not happen.

## Takeaway

On these traces, a slow, non-decaying ±2 hill climber on `prot_pct` cannot pick out the split's effect on hit rate from phase changes in the trace and from the window climber's own moves. It does worse than the fixed 75% on both web_0 and hm_0 at 131072, even on web_0, which it started out favouring. Two things stand out: the protected split is less sensitive than the window size, and grows and shrinks take effect at different speeds. So adapting the split needs a cleaner signal, such as a shadow-cache or ghost-hit comparison of protected against probation, or a step that decays to a stop. Simply aggregating more window-climber rounds is not enough.
