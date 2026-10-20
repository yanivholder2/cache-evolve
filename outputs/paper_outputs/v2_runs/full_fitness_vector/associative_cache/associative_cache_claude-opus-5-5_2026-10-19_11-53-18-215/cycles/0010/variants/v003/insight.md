# Variant v003 Insight

## Helped

- per-op-time (get): The median is flat, but the tail dropped sharply. That was the cost the parent's insight blamed on promotion. A probation hit now rotates only `items[pos..t]` / `tags[pos..t]`, ending at protected's midpoint, where it used to rotate `items[pos..]` all the way to protected MRU. With protected at 14/30/60 slots in the larger buckets, that roughly halves the longest memmove on the get path.
- per-op-time (put): The tail also fell, and the median edged down. A resident-key put goes through the same `hit()`, so a put that lands on a probation item gets the same shorter promotion rotate. The cold-key and ghost-hit insert paths did not change.

## Hurt

None of the dimensions got worse. hit-rate is essentially flat, a gain within noise. That is far less than the hypothesis promised. metadata-size did not move, because midpoint placement is purely positional: no counters, and `Item` stays 16 bytes.

## Takeaway

Splitting protected into a two-level ladder, so that once-promoted keys sit below keys with three or more hits, barely changed which keys survive on this trace. Keys with exactly two hits evidently weren't pushing hot keys out of protected, so the hit-rate thesis does not hold. What the change did do is cut the promotion rotate roughly in half, which recovered most of the get/put tail latency that the 15/16 cap had added. It costs nothing in hit rate, so it is worth keeping as a latency fix. It also suggests that reordering inside protected is a weak lever for hit rate here.
