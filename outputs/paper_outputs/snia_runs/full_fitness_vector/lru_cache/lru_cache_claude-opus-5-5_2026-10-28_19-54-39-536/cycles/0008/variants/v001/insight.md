# Variant v001 Insight

## Helped

- hit-rate (hm_0/131K): Cutting the ghost at 131K from 65536 to 32768 slots shortens the reprieve window to about capacity/4 evictions. Fewer of hm_0's long-loop keys are still in the ghost when they come back, so fewer get a pointless one-shot probation reprieve. hm_0/131K recovered past the parent's dip, which suggests most of that leak came from reuse distances between capacity/4 and capacity/2.
- hit-rate (summary): The summary rose slightly because hm_0/131K gained a bit more than web_0/131K lost. Both 16K points match the parent exactly, which confirms that only the `capacity > GHOST_MAX_CAPACITY` branch of `ghost_len_for` changed.
- per-op-time (get and put, p50 and p99): All four latencies dropped noticeably. That fits the hypothesis's idea that halving the hidden ghost table (128 KB to 64 KB at 131K) reduces cache pressure, at least on the put path, which reads and writes ghost slots on vacant inserts. `get`/`on_hit` were not touched, and the gain is larger than a 64 KB footprint cut would plausibly explain. So part of it is probably run-to-run noise or a layout shift. The parent's insight also saw unexplained per-op-time swings.

## Hurt

- hit-rate (web_0/131K): web_0/131K gave back a small part of the gain the parent's capacity/2 ghost earned. Some of web_0's useful re-references come back between capacity/4 and capacity/2 evictions, and those now fall outside the window and lose their reprieve.

## Takeaway

At 131K, hm_0 and web_0 pull the ghost length in opposite directions, but only by small amounts. capacity/4 is a slightly better trade than capacity/2 on hit-rate, and it halves the ghost's untracked heap cost. Because the hit-rate moves are now marginal in both directions, further tuning of the ghost length has little left to win. A bigger next step would be to change how reprieves are granted at large sizes, rather than to keep fine-tuning the window. `metadata-size` still does not count the ghost table, so its 64 KB per cache is a hidden cost.
