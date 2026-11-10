# Variant v001 Insight

## Helped

- hit-rate (web_0 @ 16384): The one-time relap gives a fresh freq-1 small tail one more pass through small while main is full. One more hit then graduates it (`lap == 1 && freq >= 1`). Before, it needed freq 2 or was ghosted. On web_0, a single-reuse key often gets its next reuse soon enough to land during that extra pass. Because weak ghost returnees are marked lap 2, the parent's web_0 gain from the freq-gated ghost path is kept.
- hit-rate (web_0 @ 131072): This cell gained the most. The hypothesis expected the effect at 16384, so this was not predicted. A larger small queue makes the relap pass longer, so a relapped key has more time to collect its single extra hit before it reaches the tail again. So the relap pays off more as small grows, not less.
- hit-rate (hm_0 @ 131072): Only a small gain, for the same reason as web_0 @ 131072: the longer relap window catches a few more second reuses.

## Hurt

- hit-rate (hm_0 @ 16384): The hypothesis predicted a gain here, but this cell did not move. A likely cause is a side effect of the relap. It resets freq to 0, so a key that relaps and then misses its extra hit is ghosted with freq 0, where the parent ghosted it with freq 1. When that key returns, it takes the weak-returnee path into small instead of the parent's direct ghost→main admission. On hm_0, ghost returns are strong evidence (see the parent insight), so keys lost this way cancel out what the relap recovers.

## Takeaway

Combining the relap with the freq-gated ghost path works on web_0, and the lap-2 guard keeps the two mechanisms from both protecting the same keys. The gain grows with cache size because a larger small queue gives a relapped key more time to be reused. The relap does not help hm_0 @ 16384. The likely cause is that resetting freq to 0 on relap erases the "reused once" signal that the ghost gate depends on. The next change to try: when a relapped key fails, ghost it with freq 1 (or record that it was relapped), so it keeps the direct ghost→main return it had in the parent.
