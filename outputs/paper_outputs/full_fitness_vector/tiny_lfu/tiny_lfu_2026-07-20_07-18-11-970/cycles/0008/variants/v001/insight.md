# Variant v001 Insight

## Helped

- metadata-size: Downgrading the sketch counter array from `Vec<Vec<u32>>` to `Vec<Vec<u16>>` halved the count-min sketch heap from 131,072 to 65,536 bytes at cap=1024, matching the goal of fitting the 8×-wide sketch within the same memory footprint as the original 4×-wide u32 sketch.

## Hurt

- per-op-time: The u16 counter change raised put p99 from 200 ns to 300 ns despite halving the sketch's heap size — the expected L2-pressure relief on the write path did not materialise, possibly because u16 store alignment or the `as u32` zero-extension casts added in `estimate()` introduced new overhead on the eviction/admission path.

## Takeaway

Switching to u16 counters at 8× width successfully halved the sketch heap as intended and left hit rate completely unchanged, confirming that counter precision does not affect admission accuracy at this width. However, the latency benefit that motivated the change did not appear — put p99 regressed instead of improving — indicating that the cost of u16 memory operations (stores in `increment`, zero-extension casts in `estimate`) offsets the L2 footprint saving on write-heavy eviction paths. The memory reduction is genuine but does not automatically translate to lower tail latency.
