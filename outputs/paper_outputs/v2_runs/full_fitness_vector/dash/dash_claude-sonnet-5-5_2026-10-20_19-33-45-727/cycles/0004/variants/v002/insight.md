# Variant v002 Insight

## Helped

- hit-rate: No movement. Admission logic was deliberately left alone and the insert-path rewrite did not change which keys are retained, so hit-rate matches the parent.
- metadata-size: No movement. The bit-packed flag layout and item size were kept, so the single-pass free-slot scan added no per-item state.

## Hurt

- per-op-time: Both get and put p50/p99 got slower than the parent, so the put-path change failed its goal of cutting per-op-time. The likely cause is that the merged duplicate-key and free-slot scan, plus the occupancy/bitmask check, adds work that buckets near capacity pay without benefit. Get latency also rose even though the change did not touch the get path. That points to code layout or cache effects, or to measurement noise. (Based on the hypothesis; the overlay source was not readable in this session.)

## Takeaway

Skipping the victim-selection routine on non-full buckets did not speed up inserts. Under a 2048-entry workload at 0.684 hit-rate, buckets are probably full most of the time, so the free-slot fast path rarely fires and the extra bookkeeping is pure overhead. Insert mechanics look like a poor lever for per-op-time here. The get-latency regression should be re-measured before it is attributed to this change.
