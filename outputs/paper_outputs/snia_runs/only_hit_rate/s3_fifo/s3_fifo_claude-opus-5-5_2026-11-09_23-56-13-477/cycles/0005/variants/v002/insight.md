# Variant v002 Insight

## Helped

- hit-rate (hm_0 @ 16384): The age gate does what the hypothesis aimed at. A freq-0 ghost returnee that comes back within about one small-queue turnover is now a tight reuse that a 25% small queue squeezed out. These go straight to main again, so they no longer pay v001's penalty of taking a small slot, pushing another key out to the ghost, and needing one more hit before graduating. This cell is now above v001's.
- hit-rate (web_0 @ 16384): v001's gain was kept and grew a little. On web_0, most freq-0 returnees come back late in the ghost window (older than `small_size` ghost insertions), so they still take v001's path back through small. The few that return quickly look like real reuses that are worth a main slot.
- hit-rate (hm_0 @ 131072): This reverses the slight downward drift v001 had at the large size. The recency window scales with `small_size`, so at this capacity it is wide in absolute terms. That lets most of hm_0's strongly reused freq-0 returnees bypass small again, while very old returnees are still filtered.

## Hurt

## Takeaway

Using each key's ghost age (`next_stamp - stamp`, which costs nothing because stamps already count small→ghost evictions) to split freq-0 ghost returnees worked. It turned v001's mixed per-trace result into a gain on both traces at the small size, with nothing lost anywhere. web_0 @ 131072 did not move, because at that size the main-vs-small routing of freq-0 returnees rarely changes which key is evicted. So the useful split is not "reused in small or not". It is "reused in small, or returned before small could have turned over once". A quick return is evidence on its own, even with no hit in small. The next obvious knob is `GHOST_RECENT_AGE_DIVISOR`: the window is now exactly one `small_size`, and it has not been tested whether a narrower or wider window (for example 2× small) helps the large size, where web_0 has stalled.
