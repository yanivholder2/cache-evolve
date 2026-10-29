# Variant v002 Insight

## Helped

- hit-rate (web_0, both sizes): v001's sketch counters never decayed, so main-LRU victims kept the frequency they built up early in the trace and beat window candidates that were popular now. Halving every counter every 10×capacity increments shrinks that old frequency, so current candidates win the strict `>` contest more often. This is v002's clearest gain, and it holds at both cache sizes.
- hit-rate (hm_0, large size): there is a small gain here, consistent with the same mechanism. With 131072 entries, the aging period is long enough that keys that really are hot keep enough count to survive each halving.

## Hurt

- hit-rate (hm_0, small size): at 16384 entries, the period is 10×capacity counted in sketch increments. A miss calls `increment` twice (get, then put), so a reset fires after fewer than 10×capacity requests. hm_0's long-term frequent keys get their counts halved before they build a lead. Halving also turns small counts into ties, often 0 vs 0. Under strict `>` a tie keeps the main victim, so the contest relies more on chance, and some keys that hm_0 reuses steadily seem to be lost.

## Takeaway

When cap-free halving is added on top of the 1% window, the hit-rate summary goes up. This confirms that v001 kept stale keys in main, and that the gain v003 lost came from the 15-cap, not from aging itself. The gain depends on the trace and the size: web_0 benefits from fast recency turnover, while hm_0 at the small size prefers a longer memory. The next knob is therefore the aging period. Two options: count it per request instead of per `increment` call, or scale it beyond 10×capacity for small caches. This vector measured only hit-rate, so per-op-time was not measured. That matters because each reset sweeps the whole `depth × width` table in one pass, which adds an O(width) latency spike roughly every 10×capacity increments.
