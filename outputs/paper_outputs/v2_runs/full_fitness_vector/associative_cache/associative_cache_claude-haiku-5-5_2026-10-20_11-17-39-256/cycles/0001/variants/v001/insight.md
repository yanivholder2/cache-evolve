# Variant v001 Insight

## Helped

- hit-rate: Changing `BaselineGeometry::layout` to return 64 buckets of 32 slots (instead of 128 × 16) for the 2048-slot budget gives each key a larger set of slots to occupy, which modestly reduced set-conflict misses on workload-a. The gain is small.
- per-op-time: `put` latency dropped noticeably, even though the hypothesis predicted it would rise. The overlay touches only the geometry seam, not the put path, so this is not a direct consequence of the change; it may be a layout effect (fewer, larger bucket allocations) or run-to-run noise and should be re-measured before being relied on.

## Hurt

- per-op-time: `get` latency rose slightly at both p50 and p99. Lookups scan a bucket linearly via `get_position`, and buckets are now twice as wide, so each get inspects more slots. This matches the predicted cost of wider buckets, though the increase is small.

## Takeaway

Widening buckets from 16 to 32 slots at a fixed 2048 capacity is a cheap win on hit-rate for this trace, with only a small get-path cost from the linear bucket scan. Because the overlay changes only `layout()` and leaves the bucket and eviction code untouched, the result isolates the bucket-width trade-off, but the unexpected put improvement means the put-side numbers are not yet explained by this change.
