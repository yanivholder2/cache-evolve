# Cycle 0005 Insight

## Cohort outcome

All three variants built on cycle 0004's v002, and each changed one lever. v001 shortened the climber's sample-period cap. v002 gave in-window reuse a +1 bonus in the duel. v003 raised the window ceiling from 40% to 75%. v003 is the nominal Winner, but it matched its parent exactly in every cell. It won only because it changed nothing; it did not improve anything. v001 and v002 lost in most cells, and they lost most on small-size web_0, the cell both were aimed at.

## Patterns

- The cycle produced diagnosis, not gains. Each one-lever test ruled out a cause of cycle 0004's small-size web_0 gain or limit. The window ceiling never binds, because the window never reaches 40%. A shorter period cap hurts. Duel bonuses for in-window reuse hurt.
- The climber is limited by how fast and how steadily it climbs, not by how high it may go. Its moves land lazily, as misses fill the window. A round shorter than the time a resize needs to fill measures a half-applied move, and flip-on-drop then reverses growth that would have paid off. The sample period should be tied to how long a resize takes to fill, not shortened for speed.
- In-window reuse carries no signal beyond the sketch. Every window hit is already counted, so a bonus counts the same evidence twice and tilts close duels toward recency. That holds for both the cycle 0004 bypass and this cycle's bounded bonus. The general lesson that soft signals beat hard ones still holds, but only if the signal is independent of the sketch.
- Strict one-shot filtering on small-size hm_0 is still fragile. Anything that adds climber noise or loosens close duels leaks a little of the gain back.
- For the fifth cycle, only hit-rate was measured. The window size the climber settles at, which three hypotheses asked for, was not logged or not read, so the main open question is still answered only by inference.

## Open questions

- Where does `win_cap` actually settle in each cell, and does it oscillate at small size?
- Does a larger step cap, with the period held at its current cap, let the window grow further on small-size web_0 without hurting hm_0?
- Would a sample period set as a multiple of capacity (fill time) beat a fixed access count at both sizes?

## Continuation

Treat cycle 0004's v002 as the effective parent; v003 is the same cache plus harmless headroom. Next, vary the step cap with the period held at its current value. Separately, try a period tied to capacity or fill time instead of a fixed count, or make flip-on-drop less sensitive (for example, require two drops in a row). Drop shorter fixed periods, ceiling increases, and any in-window-reuse bonus. Before the next cycle, read the climber's window-size log for each cell. Getting `metadata-size` and `per-op-time` reported is now five cycles overdue.
