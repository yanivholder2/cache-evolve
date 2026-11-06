# Variant v003 Insight

## Helped

- hit-rate (web_0 @ 16384, hm_0 @ 16384): Both small-size cells match the parent exactly. `freq_victim` is false for capacities ≤ 32_768, and `main_victim()` then returns the parent's probation-first victim with no estimate, so the gate held as the hypothesis required. This is preservation, not a gain.

## Hurt

- hit-rate (web_0 @ 131072): The hypothesis predicted this cell would gain, but it took the largest loss. The probation LRU has always just won a TinyLFU duel, so its estimate is at least as high as some candidate's. The protected LRU, by contrast, is the oldest re-referenced entry, and the sketch halves its counts every 16×capacity increments. So the "strictly colder" test keeps choosing the protected LRU. The keys this evicts are the long-reuse-distance keys that web_0 hits again at large sizes: their counters decayed, but they had not gone stale.
- hit-rate (hm_0 @ 131072): A small loss, not the neutral fallback the rationale expected. hm_0's hot set has high estimates near the MRU end of protected, but the protected *LRU* can still have decayed below a freshly admitted probation entry, so some reusable protected entries are evicted. Because the chosen victim is colder, the duel also admits window candidates more easily, which adds a little churn. The `victim_est <= 1` shortcut in the window-growing branch now fires more often for the same reason.
- hit-rate (summary): Both large cells got worse and both small cells stayed the same, so the net result is a strict regression from v002.

## Takeaway

The decaying count-min estimate cannot be compared fairly between the two main segments. Probation's LRU has just been admitted, so the duel guarantees it a non-trivial count. Protected's LRU is by definition the entry whose counts have had the most time to halve. The rule therefore turns into "evict protected first" in disguise, and it destroys the long-reuse residency that large-size web_0 depends on. A per-eviction frequency victim would need a recency-aware or age-normalized signal, or a margin much larger than "strictly colder", before it could separate truly stale protected entries from merely old ones. Better still, keep victim selection probation-first and adapt the segment split instead.
