# Variant v001 Insight

## Helped

- hit-rate: Shrinking the small queue from ~7% (`capacity / 14`, 146 slots at 2048) to ~5% (`capacity / 20`, 102 slots) transferred ~44 more resident slots to main, continuing to reduce one-time-access pollution without degrading the filter's ability to screen cold traffic before it reaches main.

## Hurt

## Takeaway

The monotone improvement in hit rate extends to a ~5% small queue: even at 102 slots the admission filter still rejects enough single-access traffic that the enlarged main queue retains higher-quality residents. This pushes the experimental floor one step below the ~7% level confirmed in cycle 0008, and suggests the filter's effectiveness is not yet limited by queue size in this range on a Zipfian workload.
