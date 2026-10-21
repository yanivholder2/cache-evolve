# Variant v003 Insight

## Helped

- hit-rate: Replacing strict recency ordering with a 2-bit saturating frequency counter plus a decrementing clock hand lets entries that were hit repeatedly survive a scan sweep, so one-shot scan entries (freq 0) are evicted first. This is the improvement the hypothesis targeted, and it shows up as a modest gain on the scan-heavy trace.

## Hurt

- metadata-size: Not measured for this variant, so no effect can be claimed. The fitness vector has only the hit-rate dimension. The hypothesis did remove the linked-list pointers and add a `freq: u8` per slot, but the variant gives no evidence of the net effect.
- per-op-time: Not measured either. Hits no longer touch a list, which should make them cheaper. Eviction now has a variable-length hand sweep, which could raise put tail latency. Neither effect is confirmed.

## Takeaway

Frequency-aware CLOCK gives a real but small hit-rate gain over LRU on this scan-heavy workload. The gain comes from scan resistance, not from any cheaper bookkeeping. Because the evaluation dropped the metadata-size and per-op-time dimensions, the variant can't show whether the cheaper hit path paid off, and the next evaluation should measure those dimensions.
