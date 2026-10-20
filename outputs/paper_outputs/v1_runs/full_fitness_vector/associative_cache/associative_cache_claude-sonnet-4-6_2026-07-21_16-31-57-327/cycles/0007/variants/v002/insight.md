# Variant v002 Insight

## Helped

## Hurt

- **hit-rate**: Without the 3-swap promotion loop, bucket order degrades to pure insertion order (FIFO), so frequently-accessed items age steadily toward the cold tail where they compete directly with unaccessed items inside the 8-item scan window — the second-chance flag alone cannot compensate for the loss of recency ordering when warm items accumulate at the scan site.

## Takeaway

Removing positional promotion revealed that the second-chance flag's discriminating power depends heavily on the items it actually sees: with promotion keeping hot items away from the tail, the scan reliably finds cold (counter==0) targets; without promotion, warm items age into the scan window alongside cold ones, cutting the scan's precision and costing hit rate. Per-op-time showed no measurable improvement at the 100 ns benchmark floor, suggesting the promotion-swap cost was already below measurement resolution on this workload — meaning v003's latency regression came from something else, and the swap elimination bought nothing here.
