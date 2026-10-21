# Variant v001 Insight

## Helped

- hit-rate: Halving the ghost slot count again (floor of 16 slots) did not cost hits. The result stayed level with the parent, slightly above it and within noise, so the ghost table was still larger than the workload needs.
- per-op-time: The get p50 and put p50 are about the same as the parent, with a trivial gain. Put p99 is also flat, so the smaller table did not cut put tail latency as the hypothesis predicted.

## Hurt

- per-op-time: Get p99 got much worse than the parent. Only the ghost sizing constant changed, and the get path does not touch the ghost table, so this is most likely measurement noise or cache-layout and alignment effects, not a real cost of the change.
- metadata-size: No movement. Both measured sizes (the per-node struct and the cache struct) are fixed-size and do not include the heap-allocated ghost table. This metric cannot show the memory saving the hypothesis claimed.

## Takeaway

Shrinking the ghost table a second time is safe for hit-rate on this trace, so the table has headroom. The benchmark does not measure the saving, because metadata-size ignores the ghost allocation and put tail latency did not move. Further shrinking is cheap to try but cannot be shown to pay off until a heap-inclusive memory metric exists. The get p99 jump should be re-measured before it is attributed to this change.
