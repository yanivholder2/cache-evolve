# Variant v002 Insight

## Helped

- `per-op-time`: Replacing four seeded `DefaultHasher` invocations per sketch call with a single `hash_key()` followed by four shift-and-mask derivations eliminated the three redundant key hashes, cutting the median get latency.
- `hit-rate`: The four non-overlapping 16-bit windows of a single SipHash-1-3 output provided enough row independence in the count-min sketch to preserve — and marginally improve — admission accuracy on this workload.

## Hurt

## Takeaway

Consolidating the sketch's hash work from four independent seeded hashes to one hash plus bit-slicing wins on the median get path, confirming that redundant hashing was real overhead. However, the put p50 and get p99 both regressed despite the scheme doing strictly less hashing per call, which suggests that deriving all four row positions from non-overlapping windows of a single 64-bit value weakens per-row collision independence compared to four independently seeded hashes — an effect that shows up as higher latency on write paths (which combine an increment and two estimates when the cache is full) and in tail percentiles where sketch collisions cluster.
