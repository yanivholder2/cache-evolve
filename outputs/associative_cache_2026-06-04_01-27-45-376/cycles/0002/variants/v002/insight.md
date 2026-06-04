# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Replacing v003's binary 0-vs-nonzero filter with a min-`lfu_counter` rank made the all-touched regime worse, not better — once every slot has been touched, evicting the lowest counter keeps a stale, once-recent victim while protecting old items that accumulated hits long ago, whereas v003's fallback to slot 0 still respected recency order.

## Takeaway

Inside a bucket, `lfu_counter` without any aging mechanism is not a strict improvement over LRU position once every slot is non-zero: frequency rank rewards items for hits they earned arbitrarily far in the past, so the "smooth" extension of v003's filter actually demotes recency exactly where v003 still leaned on it. The untouched regime — where v002 and v003 behave identically — is doing most of the work; the all-touched regime needs recency, not raw counter rank.
