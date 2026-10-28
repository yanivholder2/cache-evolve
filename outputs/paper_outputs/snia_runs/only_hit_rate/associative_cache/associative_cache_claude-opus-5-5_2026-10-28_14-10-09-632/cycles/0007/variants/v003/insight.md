# Variant v003 Insight

## Helped

None. The 131072-size cells for web_0 and hm_0 did not change, as expected. Their 1024-slot buckets keep a probation ring of `max_size / 4` = 256 slots, so the new floor never applies to them.

## Hurt

- hit-rate (web_0 @ 16384): This is the target cell, and it fell a little instead of rising. The extra 16 probation slots per 128-slot bucket give a never-hit block more misses to collect a second touch before eviction. But that extra lifetime caught too few of web_0's late re-references to make up for shrinking main from 64 to 48 slots. Fewer promoted blocks fit in main, so blocks that were already proven reusable got evicted sooner.
- hit-rate (hm_0 @ 16384): This cell dropped more than web_0 did. hm_0's hot set needs main capacity, and the finer ranking from ceiling 7 does not make up for losing a quarter of main's slots. The sweep ranks hot blocks above warm ones better, but with 48 slots some hot blocks still don't fit.

## Takeaway

Both refutation conditions in H03 were met. web_0 at 16384 did not beat the parent, and hm_0 lost more than web_0 gained, since web_0 gained nothing. So the ceiling-7 main did not move the probation-depth optimum above 64 slots, and the extra main space it would need is still binding at 128-slot buckets. Together with cycle 0006's bracket (64 helped, 96 lost), this suggests that a floor of 64 is a local optimum and the depth lever is finished. Further small-bucket gains have to come from what main keeps, not from giving probation more room.
