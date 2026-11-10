# Variant v003 Insight

## Helped

- hit-rate (hm_0 @ 16384): Doubling `ghost_cap` to `2 * main_size` keeps records for hm_0 keys that come back after between one and two `main_size` worth of ghost insertions. Those keys now get a ghost-return path: freq-1 keys go to main, freq-0 keys go back to small with freq 1. Before, they were first admissions with freq 0. At this tight capacity, that one-hit head start is what lets them graduate before small turns over, so this is the only cell where the larger ghost clearly pays off.

## Hurt

- hit-rate (web_0 @ 16384): web_0's late returnees (the extra ghost window) seldom come back again. The keys the larger ghost now admits as returnees take freq-1 small slots, and freq-1 ones take main slots, which push out keys with better short-term reuse. This is the pattern the H03 rationale warned about. It showed most at the small size, not at the large size it expected.
- hit-rate (web_0 @ 131072): This is the same mechanism, much weaker. At this size most useful web_0 rereferences already fall inside the old `main_size` ghost, so the extra records mainly add a few low-value promotions into main.
- hit-rate (hm_0 @ 131072): This is a slight loss. At the large size, hm_0's useful returns were already covered by the old ghost and the wide `small_size` recency window. The extra records only let a few very old freq-ghosted keys go straight into main, where they force a main eviction.

## Takeaway

The ghost is a real lever, but it works differently on each trace. A larger ghost helps only a trace whose reuse distances sit just beyond the old `main_size` limit at a small cache size (hm_0 @ 16384). Where far-back returns are noise, as on web_0 at both sizes, it hurts. The gain and the losses almost cancel, so a uniform 2× budget is roughly neutral overall. The useful information in an old ghost record is not "seen before". It is how far back the key was seen. Records in the extended zone should probably not grant the same privileges, especially direct-to-main for freq ≥ 1, that records inside `main_size` do.
