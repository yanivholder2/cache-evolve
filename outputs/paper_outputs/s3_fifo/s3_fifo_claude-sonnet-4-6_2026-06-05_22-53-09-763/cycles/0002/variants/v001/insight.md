# Variant v001 Insight

## Helped

- hit-rate: Changing `small_size` from `capacity / 10` to `capacity / 12` freed 34 more resident slots for the main queue at capacity=2048 (170 vs 204), extending the second-chance retention pool and allowing slightly more of workload-a's recurring items to survive eviction.

## Hurt

## Takeaway

Shrinking the small queue from ~10% to ~8.3% of capacity continues the upward hit-rate trajectory established in cycle/0001, but the gain is noticeably smaller than the jump achieved by the much larger 25%→10% reduction. The small queue at 170 slots still absorbs cold-start admissions without degrading the admission filter, yet the marginal value of each additional slot transferred to main is diminishing — suggesting the working set benefit of a larger main queue is beginning to plateau for this workload and capacity.
