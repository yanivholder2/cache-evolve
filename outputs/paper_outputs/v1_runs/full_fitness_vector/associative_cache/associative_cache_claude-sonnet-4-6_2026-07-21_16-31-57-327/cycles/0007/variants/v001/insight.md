# Variant v001 Insight

## Helped

_(no dimension improved relative to the parent)_

## Hurt

- **hit-rate**: Reducing promotion from 3 swaps to 1 swap slows hot-item migration toward the head, leaving recently-touched items closer to the cold tail where the second-chance scan operates — so items that have been accessed at least once are more frequently encountered by the scan and occasionally fall through to the positional-tail fallback, costing eviction quality on this workload.
- **per-op-time**: Put tail latency (p99) doubled relative to v003 — slower item migration means the 8-item second-chance scan more often finds all scanned items already flagged (lfu_counter == 1), forcing the fallback eviction path on inserts and adding overhead that the 3-swap variant avoided by keeping hot items further from the scan window.

## Takeaway

The single-swap change failed to decouple the second-chance scan's hit-rate benefit from promotion cost: the scan's eviction accuracy depends on hot items being pushed far enough from the cold tail that newly inserted (unflagged) items are reliably found first, and one swap per access is insufficient to maintain that separation on a Zipfian workload. Reducing swap count did not recover per-op-time either — the hypothesis that 3 swaps were the write-pressure bottleneck appears incorrect at these benchmark granularities, and the sole measurable effect of the change was a degradation in both eviction quality and put tail latency.
