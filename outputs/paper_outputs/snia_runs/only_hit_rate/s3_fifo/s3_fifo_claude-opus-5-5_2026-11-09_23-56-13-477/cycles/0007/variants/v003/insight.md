# Variant v003 Insight

## Helped

## Hurt

- hit-rate (hm_0 @ 16384): Halving the window sends hm_0's freq-0 ghost returnees aged between `small_size/2` and `small_size` back through small. That brings back the cost v002 had removed: each one takes a small slot, pushes another key out to the ghost, and needs another hit before it graduates. So these mid-window returns were real reuse, not noise.
- hit-rate (hm_0 @ 131072): Same cause at the large size. The window still scales with `small_size`, so halving it drops a large absolute band of hm_0's quick returns. Some of v002's recovery at this size is undone.
- hit-rate (web_0 @ 131072): The narrower window did not filter out useful pollution. It took away a few web_0 returnees that were worth their main slot, which is the opposite of what the hypothesis predicted for web_0.

web_0 @ 16384 did not move. At the small size, almost all of web_0's freq-0 returnees are already older than one turnover, so changing where the cut sits inside that turnover changes nothing for them.

## Takeaway

The hypothesis assumed web_0's harmful returnees sit in the second half of the turnover window and hm_0's useful ones sit in the first half. The results say otherwise. Returnees in the second half help on hm_0 and are neutral to slightly useful on web_0, so a half window only loses. Together with cycle 0006's finding that a 2× window hurts web_0, this brackets `GHOST_RECENT_AGE_DIVISOR = 1` (a one-turnover window) as a local optimum on both sides. The ghost-age axis should be treated as closed. Further web_0 gains at 131072 will need a different signal than how recently a key returned.
