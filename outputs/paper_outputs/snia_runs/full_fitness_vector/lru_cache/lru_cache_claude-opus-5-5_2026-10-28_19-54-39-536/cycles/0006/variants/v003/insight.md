# Variant v003 Insight

## Helped

- hit-rate (web_0 @ 131K): The capacity/8 ghost turned the reprieve back on at 131K, and web_0 moved up only slightly. The gain is far smaller than cycle 4's ungated reprieve, which suggests most of web_0's useful re-references at this size come back after more than ~16K evictions. The undersized table forgets them before they return, so the filter drops the very reuses it was meant to catch.
- per-op-time (get and put, p50 and p99): All four timings fell sharply, but this cannot be caused by the change. The only code edit is in `try_new` sizing above `GHOST_MAX_CAPACITY`. The timing benchmark runs at the small size, where `put`, `get` and `on_hit` are byte-for-byte identical to v001. So this is benchmark noise. It strongly suggests that v001's reported put/get regression was at least partly noise too, not just the cost of the extra `hash_one` calls.

## Hurt

- hit-rate (hm_0 @ 131K): hm_0 at 131K dipped slightly, so the reuse-distance filter did not fully exclude hm_0's long-loop keys. Some still land in the 16K-slot ghost through tag survival or 16-bit tag collisions and get a reprieve they don't deserve. The loss is far smaller than cycle 4's, though.

## Takeaway

The 16K results should be identical to v001, since that code path did not change. Yet hm_0 @ 16K still moved by about as much as either 131K trace did. The likely cause is that the default `RandomState` hasher is seeded differently on each run, which changes ghost slot and tag collisions. So the small 131K web_0 gain and hm_0 loss are within run-to-run noise. Shrinking the ghost to capacity/8 is a safe, roughly neutral way to turn the reprieve back on at large sizes, but it does not unlock the web_0 gain cycle 4 found. That gain seems to need a reuse window longer than capacity/8 (for example /4 or /2), and judging it needs a fixed-seed hasher so a few-basis-point difference can be told apart from noise.
