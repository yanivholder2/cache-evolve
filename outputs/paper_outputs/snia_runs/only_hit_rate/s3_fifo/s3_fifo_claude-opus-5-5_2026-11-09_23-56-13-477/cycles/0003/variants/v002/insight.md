# Variant v002 Insight

## Helped

(none: no cell improved on the parent.)

## Hurt

- hit-rate (web_0, both sizes): v001's strict freq >= 2 rule is what protected web_0. Here that protection held only while the windowed ghost-return rate stayed under 0.30, and the small regression means it didn't. On some windows web_0's rate went above 0.30 and switched the threshold to 1. That brought back the single-reuse promotions into a full main, which cost an extra `evict_main` pass each, the cost v001 had removed.
- hit-rate (hm_0, 131072): the same mechanism slipped here. The v001 insight showed that freq >= 2 was the better rule at this size, but the large cache's ghost (sized to `main_size`) is long enough that many ghosted keys return. That pushed the threshold to 1 where it doesn't pay off.
- hit-rate (hm_0, 16384): this cell matches the parent, so the adaptation recovered none of the threshold-1 benefit it was built to win back. Either the threshold barely left 2 on this trace, or switching to 1 at this size no longer helps on top of the freq reset.
- hit-rate (summary): three cells regressed slightly and the one cell the hypothesis targeted stayed flat, so the aggregate falls below v001.

## Takeaway

The ghost-return rate triggered on the wrong traces. It switched to lenient where strictness helped (web_0, hm_0 at 131072) and gained nothing on hm_0 at 16384. The likely flaw is that a high ghost-return rate shows the ghost path is already recovering those keys cheaply: a returning key costs one miss and then goes straight into main. It doesn't show that promoting at freq >= 1 would have been better. The numerator also counts hits on any key in the ghost, which holds up to `main_size` keys (about 3× the small queue), but the denominator counts only this window's ghosted keys. That makes the ratio a long-horizon reuse signal, not a measure of what strictness costs. A future gate should compare the two paths directly: ghost-return misses avoided by promoting versus main entries displaced by promoting. This variant does not show where each trace's threshold settled, because no `threshold_stats()` output was found among the eval artifacts available here. Only hit-rate was measured, so per-op-time is unknown. The new counters add per-cache state but don't change the `Entry` layout.
