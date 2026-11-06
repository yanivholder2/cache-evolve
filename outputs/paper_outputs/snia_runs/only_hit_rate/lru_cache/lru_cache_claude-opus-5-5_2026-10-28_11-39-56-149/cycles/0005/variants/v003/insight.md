# Variant v003 Insight

## Helped

None. Every trace/size cell is identical to v002, to every reported digit.

## Hurt

None. No cell moved.

## Takeaway

Raising `max_win` from 40% to 75% of capacity had no effect. In v002, the 40% ceiling never limited the window. `max_win` is read only in the `clamp` inside `climb()`. If any climb round had been cut off at the old ceiling (6553 at 16384, 52428 at 131072), the climbed `win_cap` would have differed and so would eviction order. The identical results mean the window never reached 40% in any cell. This disproves H03's premise that web_0@16384 is held back by the ceiling. The step is capped at 256 and shrinks by 2% each round, and the sample period is 50,000 accesses. Reaching 6553 from about 163 would take at least 25 straight growing rounds, which is more than 1.25M gets with no hit-rate drop. The likely limit is the climber's speed (step and period caps) or the climber reversing direction early, not how high it may go. Future work should try H01-style period/step changes, or H02-style admission changes. The 75% ceiling (and its `debug_assert!`) is harmless and can stay as headroom in case a faster climber needs it. Check the `climb` diagnostic log from this run to confirm that `max_win` was never reached and to see where `win_cap` actually settled.
