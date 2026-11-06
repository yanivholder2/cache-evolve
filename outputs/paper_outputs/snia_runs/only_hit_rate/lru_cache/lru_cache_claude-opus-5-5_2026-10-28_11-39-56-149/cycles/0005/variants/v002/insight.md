# Variant v002 Insight

## Helped

None. No trace/size cell improved on the parent.

## Hurt

- hit-rate (web_0 @ 16384): This was the cell the hypothesis targeted, and it fell the most. A window hit goes through `get()`, which already calls `sketch.increment`. So the in-window reuse that sets `WIN_REUSED` is already part of the candidate's estimate, and `cand_score` counts that one event twice. The +1 then wins duels against probation victims that are one count ahead because of longer-run frequency. That trades steady main residents for short-lived window reuse, and web_0 at small size has little room to absorb the swap.
- hit-rate (hm_0 @ 16384): A smaller drop. The bounded bonus mostly kept the hm_0 burst leak from cycle 0004 closed, as intended. But burst-then-dead blocks that get one in-window hit on top of a prior sketch count still reach `est >= 2` and qualify for the bonus. A few of them now push out probation entries that are one count hotter.
- hit-rate (web_0 @ 131072, hm_0 @ 131072): Marginal drops. At this size, close duels where a one-count bonus changes the outcome are rare. The small loss fits the same mechanism: a few warmer victims are swapped for window-reused candidates. The changed duel outcomes may also shift the climber's hit-rate samples a little, but the cache doesn't log `win_cap`, so this can't be confirmed.

## Takeaway

In-window reuse is not independent of the sketch. Every window hit is already counted by `get()`, so adding a bonus for the same hit counts that evidence twice instead of adding new signal. Even as a +1 bonus limited to estimates of at least 2, it tilts close duels toward recency, and it loses on both traces, worst on web_0@16384, the cell it was meant to fix. If in-window reuse is going to help, it has to carry information the sketch doesn't have. Two options: reward only reuse that spans a minimum gap of window time, or use the flag to break exact ties against the candidate rather than to add a point. Stacking it on the sketch estimate just repeats the window hit.
