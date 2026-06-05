# Variant v003 Insight

## Helped

- hit-rate: Multiplicative decay (0.75× per cycle) is gentler than right-shift division (0.5×), allowing frequently accessed items to retain discriminative frequency scores longer and avoid premature eviction during workload phase shifts.

## Hurt

## Takeaway

Smoothing the decay curve from aggressive (0.5×) to gradual (0.75×) multiplicative decay provides marginal but consistent improvement in hit-rate. The change preserves frequency discrimination across reset cycles: items maintain higher relative frequencies longer, which helps the cache retain items through phase transitions without losing track of their historical access patterns. The improvement is modest (0.33%), suggesting the workload's phase shifts are subtle enough that even gentler decay provides sufficient forgetting to adapt—the algorithm now balances recency and frequency better by avoiding the information loss that comes from repeatedly halving all counters.
