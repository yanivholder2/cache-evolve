# Variant v001 Insight

## Helped

- **hit-rate**: Reducing the small-queue admission threshold from 1/5 to 1/6 of capacity further tightens frequency selectivity, ensuring only items demonstrating stronger reuse patterns graduate to the main queue, which concentrates cache capacity on the highest-confidence items and improves hit rate.

## Hurt

(none)

## Takeaway

Continuing the selectivity-strengthening trajectory from v003, the 1/6 threshold demonstrates that the workload rewards even tighter admission gates. The smaller small queue reduces wasted space on items that fail to prove reuse, allowing the main queue to remain denser with reliable repeaters. This validates that the current workload remains below the utilization cliff—stricter filtering continues to pay dividends without constraining the queue so severely that working-set diversity suffers.
