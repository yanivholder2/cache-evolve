# Variant v001 Insight

## Helped

- hit-rate (web_0 @ 16384): Some freq ≥ 1 ghost records are older than `main_size` but still inside v003's doubled ghost. These now go back to small with freq 1 instead of straight into main. web_0's late returnees seldom get reused again, so most of them leave small without displacing main residents, and part of v003's main-pollution loss comes back.
- hit-rate (web_0 @ 131072): This is the same mechanism, barely visible. At this size few useful web_0 returns land in the outer tier, so taking away their main privilege changes little.
- hit-rate (hm_0 @ 131072): This is a slight recovery of v003's small loss at the large size. The very old freq-ghosted keys that v003 sent straight into main, forcing a main eviction, now have to prove themselves in the wide small queue first.

## Hurt

- hit-rate (hm_0 @ 16384): Part of v003's gain in this cell came from outer-tier freq ≥ 1 returnees going directly to main. Under tiering they re-enter small with freq 1. With main full, they need freq 2 to graduate, so they need another hit before the tight small queue turns over. Many hm_0 keys don't get that hit in time, so they are ghosted again rather than kept.

## Takeaway

The tier boundary at `main_size` moves hit-rate in opposite directions on the two traces. The privilege it removes, direct-to-main for freq ≥ 1 records aged between `main_size` and `2 * main_size`, was costing web_0 but was a real part of what the doubled ghost gave hm_0 at the small size. So v003's hm_0 gain did not come only from saving keys from a cold freq-0 admission. Ghost age alone, at this boundary, does not separate useful late returnees from noisy ones. The net effect is about neutral to slightly negative compared with the untiered parent.
