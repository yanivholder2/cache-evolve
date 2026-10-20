# Variant v001 Insight

## Helped

- per-op-time: The put tail moved slightly in the right direction, which is consistent with the aging step being a single counter increment and compare on the insert path. The margin is small and could be noise.

## Hurt

- hit-rate: Halving every main-queue frequency counter once per capacity's worth of inserts cools entries that were hot, so they become evictable sooner than the second-chance rotation would have evicted them. On workload-a at 2048 this lost hit rate instead of recovering it.
- metadata-size: The new aging counter is a `u32` field on the cache struct, and the `S3Fifo` container footprint grew by about one aligned word as a result. The `Entry` size is unchanged, so the summary metadata figure does not show this cost.
- per-op-time: Get latency rose at both p50 and p99 even though the hypothesis left `get` untouched. This is more likely a layout or codegen side effect of the struct change than a cost on the hit path, so it should not be read as the aging logic itself.
- per-op-time: Put p50 rose, which fits the amortized aging pass on inserts: the periodic walk over resident main entries adds work on a fraction of puts even though it is O(1) on average.

## Takeaway

Periodic halving of main-queue frequency counters reduced hit rate on workload-a at 2048, so in this lineage aging made eviction more aggressive rather than less. The hypothesis assumed stale hot entries were the bottleneck, but the result suggests the second-chance budget was already retaining the right entries, and halving them throws away useful frequency signal. Any future aging attempt should test a much longer period or halve only entries that have gone unhit for a full aging window.
