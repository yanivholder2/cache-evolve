# Variant v003 Insight

## Helped

## Hurt

## Takeaway

This variant made no policy change, and hit-rate matching v001 exactly confirms that. That result is also the correctness check the hypothesis relied on. It shows that the `MIN_RESIDENT_FREQ = 1` floor holds: counters never decay, and every resident was counted at least once, either by its `put` or by the `get` miss that the `pending_miss_hash` dedup stands in for. With the floor holding, the early stop in `select_victim()` and the early rejection of newcomers whose estimate is ≤ 1 can skip work without changing which victim is chosen or whether the newcomer is admitted. The cached `Node.hash` also stayed consistent through slot reuse.

The goal was to cut per-op time: `select_victim` no longer rehashes up to 8 residents, one-hit-wonder newcomers skip the whole scan, and the victim key is no longer cloned. But the fitness vector still scores only hit-rate, so this cycle cannot say whether any of that paid off.

These shortcuts are only safe while there is no aging, because aging would let a resident's estimate fall below 1. Any later hypothesis that adds counter aging or reset must remove or re-derive the ≤ 1 shortcuts.
