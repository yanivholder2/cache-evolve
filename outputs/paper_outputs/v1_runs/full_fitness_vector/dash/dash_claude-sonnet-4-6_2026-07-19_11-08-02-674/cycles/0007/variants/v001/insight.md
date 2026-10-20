# Variant v001 Insight

## Helped

_(no dimensions improved)_

## Hurt

- **hit-rate**: The clock pre-pass selects the first item to reach `lfu_counter == 0` in positional rotation, whereas the parent's global-minimum scan always found the single least-frequently-used item in the bucket; this less precise victim selection causes genuinely cold items at unfavorable positions to survive eviction while items with slightly higher (but still low) counters are evicted, reducing hit rate on a Zipfian trace where the difference between the true minimum and the positional minimum matters.
- **per-op-time**: Each step of the clock pre-pass calls `rotate_left(1)` — an O(n) Vec shift — for every warm item it passes, so in buckets where many items have non-zero counters the pre-pass performs O(k·n) work before finding a victim; the parent's LFU scan was a single O(n) read-only pass with no mutations, explaining the p99 get latency increase.

## Takeaway

Grafting the two-chance clock onto v001's promotion stack failed because v001's `lfu_counter` values, while non-zero for warm items, are not uniformly distributed: the clock's counter-decrement side-effects corrupt frequency state mid-eviction and its early-exit selection is no substitute for the parent's globally optimal minimum scan. The clock's O(1)-amortized advantage only materializes when the cold tail dominates instantly (counter == 0 at position 0); when buckets are warm throughout, the clock degrades to an O(n²) rotation that simultaneously raises p99 latency and evicts suboptimal victims. Replacing a precise global scan with an approximation requires the approximation to be nearly as selective, which the clock is not in this counter-rich environment.
