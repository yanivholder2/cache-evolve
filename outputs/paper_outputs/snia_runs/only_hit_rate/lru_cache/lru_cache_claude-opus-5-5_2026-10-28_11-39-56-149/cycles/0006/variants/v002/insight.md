# Variant v002 Insight

## Helped

- hit-rate (web_0 @ 131072): A gain within noise. At this size the 256-entry step is tiny relative to the cache and the window has a lot of room. Delaying a flip by one 50,000-access sample therefore changes the window very little, so this cell can't really tell the two flip rules apart.
- hit-rate (hm_0 @ 16384): Effectively flat. The sketch's one-shot filtering and the `min_win` floor are unchanged, and those are what mainly protect this scan-heavy cell. The flip rule probably moved the window only within a range where hit rate barely changes.

## Hurt

- hit-rate (web_0 @ 16384): This was the cell the hypothesis expected to improve most, and it dropped the most. At this size every round moves the window by up to 256 entries (about 1.6% of capacity). Requiring a second drop before reversing adds a full extra round of movement in the losing direction, and then another round to undo it. Single drops on this trace were evidently real signals, not resize noise. The parent's flip-on-first-drop was tracking them correctly.
- hit-rate (hm_0 @ 131072): A small loss on a cell the hypothesis expected to stay flat. The two-drop rule lets the window keep growing for one more sample period after the hit rate first falls. On this scan-heavy trace, the extra window space admits scan traffic that the 1%-anchored main area would otherwise have filtered out.
- hit-rate (all cells): The counter resets on any sample that is `>=` the *previous* sample, not the best one. A slow decline with small rebounds (A > B, then B <= C < A) therefore never builds two drops in a row. The climber can drift in a losing direction indefinitely, and the parent never had this failure mode.

## Takeaway

Hysteresis on the reversal decision made the climber worse, especially on small-size web_0, the case it was meant to fix. The single-sample drops it was meant to filter were mostly real, and they are the climber's only steering signal. So these results don't support the cycle-0005 "lazy-resize transient" diagnosis at the current 50,000-access period. A half-applied resize either rarely causes a false drop, or the drop still points the right way. If reversal noise is still suspected, a better next lever is a dead-band, for example flipping only when `hr < prev - epsilon`, or comparing against a smoothed rate. These filter tiny drops without delaying reactions to real ones. Future variants should also log `win_cap` per cell, which would confirm whether web_0@16384 overshot.
