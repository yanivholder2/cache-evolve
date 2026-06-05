# Variant v002 Insight

## Helped

- **hit-rate**: Two-pass victim selection ensures that when items share the minimum frequency, the oldest (head-most) entry in recency order is evicted, not an arbitrary scan-order choice. This tighter alignment between eviction order and true LRU position captures temporal locality within frequency tiers more accurately.

## Hurt

(none)

## Takeaway

Decoupling frequency-finding from victim-selection via a two-pass scan proves effective on this workload. Rather than accepting the first item that happens to have minimum frequency during a left-to-right traversal, explicitly selecting the head-most item with minimum frequency guarantees genuine LRU tie-breaking independent of memory layout. This structural clarity yields measurable improvement in cache hit-rate by better respecting access patterns within equal-frequency groups.
