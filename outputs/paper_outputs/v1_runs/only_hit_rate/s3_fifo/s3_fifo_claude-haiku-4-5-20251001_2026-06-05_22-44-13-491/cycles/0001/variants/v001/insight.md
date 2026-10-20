# Variant v001 Insight

## Helped

- hit-rate: Extending MAX_FREQ from 3 to 4 allows moderately-reused items in the main queue one additional eviction round before permanent discarding, capturing more value from sustained but not-hot reuse patterns.

## Hurt

(none)

## Takeaway

The baseline's MAX_FREQ=3 was slightly too aggressive in evicting items that still showed moderate reuse. Adding one more frequency level to the second-chance mechanism provides measurable hit-rate improvement by retaining items that exhibit sustained (but not frequent) access patterns. This suggests S3-FIFO benefits from tuning its frequency threshold based on workload reuse depth.
