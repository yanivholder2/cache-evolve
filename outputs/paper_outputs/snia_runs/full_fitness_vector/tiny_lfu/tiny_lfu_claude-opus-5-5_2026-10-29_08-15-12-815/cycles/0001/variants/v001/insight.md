# Variant v001 Insight

## Helped

- hit-rate: Every newcomer now goes into the 1% LRU window, so new blocks are kept long enough to be hit again. The parent rejected any newcomer that did not out-score the LRU victim, which meant new blocks were dropped on first sight. The gain is largest at the 16384-entry size on both web_0 and hm_0, where the parent's filter was hurting the most. At 131072 entries the gain is much smaller.
- hit-rate: The 80% protected segment keeps blocks that have been hit more than once away from the probation churn. Only probation entries (or protected entries, if probation is empty) can lose the frequency contest, so a burst of window evictees cannot push out the proven-hot working set.
- per-op-time (put p99): The tail of `put` got shorter. On a full cache, `put` now does one sketch comparison and one relink, then writes the newcomer into the slot it just freed. One plausible reason this beats the parent's admit/reject path is that the fixed "pick the evictee first, then reuse its slot" order removes a costly branch. That is a guess, not something the benchmark shows.

## Hurt

- metadata-size: `Node` gains a `region: u8` tag. Alignment pads that to a full word, so every entry costs 8 more bytes. The cache struct also grows, because it now holds three `RegionList` head/tail/len records plus the `window_cap`, `main_cap` and `protected_cap` fields.
- per-op-time (get): A hit is no longer a single move-to-MRU. `on_hit` now branches on the node's region, and a probation hit moves the entry into protected. If protected is then over its cap, the hit also demotes protected's LRU back to probation, which is a second unlink/push. The p99 cost of `get` rises noticeably as a result, and p50 rises slightly.

## Takeaway

The parent's main weakness on MSR block traces was its strict admit-or-drop rule for newcomers, not the sketch itself. A 1% window plus a probation/protected split fixed this without touching the sketch's size or layout. The benefit shrinks as the cache grows, because large caches already have enough slack to absorb bursts of new blocks. The cost is a fixed 8 bytes more per node and a heavier `get` tail. Future variants could try to remove the region byte, for example by packing it into a spare bit, or to tune the window share for the small cache sizes, where the gain is concentrated.
