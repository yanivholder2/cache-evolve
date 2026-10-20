# Variant v003 Insight

## Helped

- hit-rate: The hypothesis reshaped the CountMinSketch from its baseline depth to one fewer row, with more columns at the same counter-byte budget, so each counter is shared by fewer colliding keys. The only measured trace is workload-a at size 2048, so this is a small, single-point gain.
- per-op-time: Get and put latencies both dropped against baseline. The hypothesis only changed the sketch modulus, so this is not a direct consequence of the stated change; it may reflect measurement noise or a layout effect of the new shape, and should be treated as unconfirmed.

## Hurt

- None. metadata-size is unchanged in its summary value (the Node footprint is the same), and the sketch allocation is 4 bytes under the baseline budget, consistent with the fixed-memory constraint.

## Takeaway

Trading one sketch row for extra columns at a fixed counter budget produced a small hit-rate gain on workload-a at size 2048 without adding metadata, which supports the hypothesis that collision spread, not sketch size, limits frequency estimates here. One trace and one cache size is thin evidence, so the gain needs confirmation on other workloads before it is relied on. Note: the source overlay and manifest were not readable in this session, so the resulting row and column counts are not recorded here.
