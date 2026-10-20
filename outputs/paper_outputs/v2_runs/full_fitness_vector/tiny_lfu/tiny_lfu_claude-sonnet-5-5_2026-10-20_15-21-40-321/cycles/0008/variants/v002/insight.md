# Variant v002 Insight

## Helped

- per-op-time (get): the get path is unchanged. Resizing the window only moves the split constant, so get latency stays level with the parent.

## Hurt

- hit-rate: The hypothesis was refuted. Shrinking the window to `capacity / 20` did not beat the parent's hit-rate and came out slightly below it. A smaller window gives less room for recency-heavy items to build frequency before the `incoming_freq > victim_freq` test. On this trace the parent's larger window was already at or near a good operating point.
- metadata-size: The per-`Node` size and the cache struct both grew from the parent. The hypothesis said no new state would be added and the compact Node layout would be left alone, so this does not follow from the split change. I could not inspect the overlay sources in this session, so I can't say what caused it. It may be a layout or field change that slipped in with the edit. It needs checking against the diff.
- per-op-time (put): put p50 and especially p99 got worse. A smaller window means more puts overflow it and go through the admission and eviction path. A larger Node adds some cost too, though I can't confirm that cause. The "speed stays flat" prediction held for get but not for put.

## Takeaway

Window size is not an untapped hit-rate lever for this trace. Moving it down to 5% lost a little hit-rate and made the put path slower, because more inserts go through the main-region admission and eviction path. The unexpected Node and struct growth means the edit was not confined to the split constants, so anyone building on this variant should audit the overlay diff before reusing it.
