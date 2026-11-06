## Recurring wins

- Only one cycle is complete, so nothing has recurred yet. The items below are candidate patterns that still need confirming in later cycles.
- The changes that pay off target TinyLFU's admission contest. They change which keys compete (an admission window), which resident gets evicted (a probation segment), or how fresh the frequency evidence is (sketch aging). Changing eviction order alone has not been the lever.
- In cycle 0001 all three W-TinyLFU components helped on their own. The small LRU admission window gave the largest and most consistent gain.
- Gains are largest at small cache sizes and shrink at large sizes, most clearly on recency-friendly traces such as hm_0.

## Recurring failures

- Capping sketch counters low and admitting only on strictly greater frequency is a likely failure. Many hot keys tie at the cap, newcomers get rejected, and this produced the only regression seen so far (web_0, large size). One cycle is not enough to call it recurring.
- Every result so far measured hit-rate only. The metadata-size and per-op-time costs of the winners are unknown, so no win can yet be treated as free.

## Open questions

- Do the window, SLRU and aging gains add up when combined, or do they overlap? For example, the window may already recover the small-size gains from aging.
- Does aging without the low counter cap, or with a higher one, remove the large-size regression? Should the aging period be set separately from capacity?
- What window size is best? Should it adapt to capacity, given that the 1% window's gain shrinks as capacity grows?
- What do region tags, a second list and sketch resets cost in metadata and per-op time?

## Per-policy notes

- **Baseline TinyLFU:** its weak point is that once the cache is full it rejects every first-time key. Fixing admission matters more than fixing eviction order.
- **W-TinyLFU (window, v001 lineage):** current winner, with the strongest gains at small sizes. This is the base to build on: add SLRU and uncapped aging to it one step at a time.
- **SLRU main region (v002):** gained most on scan-heavy, one-hit traces (web_0).
- **Sketch aging (v003):** helps overall, but its large-size regression on web_0 is likely caused by the counter cap tied with strict `>` admission, not by the aging itself.
