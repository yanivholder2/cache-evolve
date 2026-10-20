# Variant v002 Insight

## Helped

## Hurt

- per-op-time: The put tail rose against the parent. The ghost's flat table uses linear probing and backward-shift deletion on full-ghost inserts, which adds probe work on the put-miss path. That work lands in the tail, not the median, which fits the observed shape.

## Takeaway

Hit rate held exactly, as H02 predicted, since the ghost's fingerprints and eviction order are unchanged. The predicted put-miss median gain and footprint reduction did not show up. The container-size probe measures inline struct size, so a heap-allocated table would not register there, and the metadata dimension is silent on this change. The one put-side effect that did show is a tail regression, so the next step is to measure ghost heap bytes directly and profile the put-miss path before judging the table.
