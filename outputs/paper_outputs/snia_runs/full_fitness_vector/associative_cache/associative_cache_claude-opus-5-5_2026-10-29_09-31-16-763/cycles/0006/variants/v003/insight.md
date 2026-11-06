# Variant v003 Insight

## Helped

- hit-rate (hm_0, small cache): This is the only cell that moved the way the hypothesis predicted. At 16K, buckets turn over fast, so the reuse bit is cleared at eviction often enough to still mean "hit recently". Sparing those items at the LRU end kept some of hm_0's reused keys from being pushed out by the next scan admission.
- per-op-time (put p50): The typical put got a little faster. In the common case the second-chance loop just checks `items[0].lfu_counter` and exits right away, so the change adds almost nothing to the median miss path. Get is unchanged, which shows the extra `lfu_counter = 1` store on the hit path costs nothing measurable.

## Hurt

- hit-rate (hm_0, large cache): At 128K, items stay resident long enough that almost all of them have been hit at least once. The bit is set on every hit but only cleared when the item is spared, so it saturates. Then the loop spares the two oldest items, which are reused but stale, and evicts the third-oldest. That item is just as likely to be reused, so the policy gives up some of plain LRU-end ordering and gets no better reuse signal in return.
- hit-rate (web_0, both sizes): Scan resistance mostly held, because scan keys never get the bit, but both cells slipped slightly. When stale reused items are rotated to `(len-1)/4`, they sit in the same probationary region where new admissions go in at `len/4`. That crowds the region and moves fresh items closer to eviction.
- per-op-time (put p99): The tail of put got clearly worse. When the bits are saturated, an eviction does two `rotate_left` calls over about len/4 `Item`s and the same over `tags`, and on wide buckets that memmove shows up in the tail. The hypothesis's bound ("≤ 2 rotations of ≤ len/4") holds, but it is not cheap.

## Takeaway

A 1-bit reuse marker that is set on every hit and cleared only at eviction time loses its meaning once the resident set has mostly been hit, which happens at large cache sizes. The second chance then becomes reordering near the LRU end with no real signal behind it, and it adds rotation cost to the put tail. If this approach is tried again, the bit needs to decay independently of eviction (for example, clear it on a sweep or when the item passes quarter depth), or the spared item should go somewhere that doesn't compete with new admissions. Raising `SECOND_CHANCE_MAX` would only make the saturation worse.
