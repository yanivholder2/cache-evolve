# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Replacing the queue scans with a `resident` key index kept hit-rate exactly at the parent's value, which matches the hypothesis's claim that eviction order is preserved: the small/main queues now hold only keys and the second-chance and ghost logic is unchanged. The cost-side claim (per-op-time down, metadata-size up) is not tested by this evaluation, because the variant's fitness vector reports only hit-rate while the baseline reports metadata-size and per-op-time. The next cycle needs those dimensions re-measured for this variant before the O(1) claim can be credited or refuted.
