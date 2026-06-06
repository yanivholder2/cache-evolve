# Variant v002 Insight

## Helped

- hit-rate: The hybrid frequency-recency scoring metric prioritizes evicting cold items with low frequency estimates while protecting recently-accessed hot items, better balancing temporal and frequency information than pure LRU ordering.

## Hurt

(none)

## Takeaway

The shift from pure LRU eviction to a hybrid metric that scores candidates by (recency / frequency + 1) successfully improves hit rate on the workload. This demonstrates that incorporating frequency information directly into victim selection—even with a simple arithmetic formula—outperforms strict age-based ordering, and that protecting hot items through both high frequency and recent access is more effective than age alone.
