# Variant v003 Insight

## Helped

- hit-rate: New misses are inserted at the eviction-next position instead of MRU, so one-shot entries leave the bucket before they can displace the working set on the workload-a trace.

## Hurt

## Takeaway

Changing only where new entries start in the bucket ranking, leaving victim selection and hit handling untouched, is enough to move hit-rate on this scan-heavy trace. The fitness vector reports only hit-rate for this variant, so the metadata-size and per-op-time effects of the insert-position flag and miss counter are unmeasured here, even though the hypothesis claims no new per-entry metadata.
