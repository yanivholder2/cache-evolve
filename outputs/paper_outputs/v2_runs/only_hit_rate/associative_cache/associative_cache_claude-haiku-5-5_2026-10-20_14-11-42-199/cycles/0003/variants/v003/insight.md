# Variant v003 Insight

## Helped

- hit-rate: Eviction draws from the probationary region first, so single-use miss inserts are displaced before entries that have already been hit again, which is the scan-resistance effect H03 targets on workload-a.

## Hurt

## Takeaway

Segmenting each bucket by reuse and evicting from the probationary region first improves hit-rate on workload-a, which supports scan resistance as a lever distinct from changing the ranking within a single pool. The fitness vector only reports hit-rate, so the cost of the second region in metadata and per-op time is still unmeasured.
