# Variant v001 Insight

## Helped

- hit-rate: Maintained 0.6611 because TimestampLRU preserves the recency-based eviction strategy regardless of bookkeeping method.
- metadata-size: Item size remains 40 bytes because TimestampLRU only replaces the implementation detail of tracking recency, not the data structure itself.

## Hurt

- per-op-time: Both get (69→100ns p50, +45%) and put (143→200ns p50, +40%) latencies increased, contradicting the hypothesis that O(1) timestamp updates would outperform O(n) vector removals and appends on 16-item buckets.

## Takeaway

TimestampLRU's theoretical O(1) per-hit cost vs. ClassicLRU's O(n) complexity did not yield the expected latency improvement. This reveals that algorithmic complexity alone does not predict CPU-level performance: ClassicLRU's vector manipulations on small bucket sizes (16 items) apparently had better cache locality or compiler optimization than TimestampLRU's additional memory accesses to update timestamps. The change teaches that constant-factor overhead differences matter more than asymptotic complexity when working with small, cache-resident data structures.
