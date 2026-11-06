# Variant v003 Insight

## Helped

- hit-rate (web_0 @ 131072): This is the largest gain the lineage has had in a single cell since probation was added, and it passes the hypothesis's bar easily. In 1024-slot buckets the window covers the newest 16 probation insertions. That is wide enough to catch web_0's back-to-back and burst re-references, which until now earned a main slot from a single early hit. Those blocks now reach the FIFO head with counter 0 and are evicted, so they no longer displace real main residents. This supports cycle 0007's finding that never-re-hit, counter-0 promotions made up most of main's victims.
- hit-rate (hm_0 @ 16384, hm_0 @ 131072): Both cells rose slightly instead of falling. This answers the refutation clause's concern: on hm_0, short-gap reuse is not real reuse. Its long-distance re-references arrive after the window closes and still promote, and the few burst promotions removed from main free a little space.

## Hurt

- hit-rate (web_0 @ 16384): Flat to slightly down, so it misses the hypothesis's bar at this size. In 128-slot buckets the window is only `64 / 16 = 4` insertions. Most of a burst's later hits land after the window and still bump the counter and promote. So the filter catches almost none of the correlated promotions here, while it occasionally withholds the bump from a true immediate re-use.

## Takeaway

Filtering correlated references is the first change to probation's *promotion* signal that has worked, and its effect scales with the window's absolute width, not with `small_len / 16` as a ratio. At 16 slots it removes burst-driven promotions on web_0 without costing hm_0 anything. At 4 slots it is too narrow to matter. The obvious next step is to give 128-slot buckets a wider window, for example an absolute floor of about 16 slots or a smaller divisor, rather than drop the idea. The hm_0 result shows there is headroom before real reuse starts being filtered.
