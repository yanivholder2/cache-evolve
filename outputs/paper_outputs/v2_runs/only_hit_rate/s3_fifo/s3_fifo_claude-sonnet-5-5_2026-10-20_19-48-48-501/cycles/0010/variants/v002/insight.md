# Variant v002 Insight

## Helped

- hit-rate: shrinking the small queue to about 2% of capacity (main takes the remainder) produced a slight gain on workload-a at size 2048. A little more main space and a quicker small-queue turnover let fewer reused entries get evicted before they could be promoted.

## Hurt

- (none) No dimension regressed. Hit-rate is the only measured dimension.

## Takeaway

Moving the small queue from the parent's sizing down to about 2% is a nearly flat change that is slightly positive. The trend of smaller small queues is therefore flattening on this trace, with the optimum at or just below this range. Further shrinking is unlikely to give meaningful gains, and a different lever such as promotion rules or ghost sizing is probably worth more. This is inferred from the hypothesis and the fitness vectors, because the overlay source was not readable here.
