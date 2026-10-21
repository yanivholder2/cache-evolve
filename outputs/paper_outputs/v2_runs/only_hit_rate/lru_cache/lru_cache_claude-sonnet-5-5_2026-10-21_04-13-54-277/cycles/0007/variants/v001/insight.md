# Variant v001 Insight

## Helped

- hit-rate: Choosing the coldest of the 4 oldest probation entries by sketch estimate, instead of always the LRU head, spares recently-hot-but-old entries and evicts a genuinely low-frequency one. The sketch frequency now persists across segments, so the scan has a real signal to use. The gain over the parent is small.

## Hurt

- (none) Hit-rate is the only measured dimension, and it did not regress.

## Takeaway

A bounded 4-deep frequency-aware victim scan works once the sketch carries persistent frequency, but the improvement is marginal. The probation-LRU head was already a near-optimal victim on this trace, so most of the available gain was probably captured by the earlier admission and segmentation changes. Widening the scan would likely yield diminishing returns. I did not check the code diff, so I can't say whether the scan adds per-insert cost.
