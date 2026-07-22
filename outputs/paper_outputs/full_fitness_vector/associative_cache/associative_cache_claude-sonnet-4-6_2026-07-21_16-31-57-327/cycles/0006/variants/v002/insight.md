# Variant v002 Insight

## Helped

## Hurt

- **hit-rate**: Halving the bucket count forces more distinct keys to share each bucket, so intra-bucket eviction pressure rises faster than the added depth can absorb it — and the 3-swap promotion cap means items landing beyond position 3 in the now-wider buckets can never reach the protected head in a single access, making them effectively invisible to the recency policy.

## Takeaway

Wider buckets hurt hit rate on this workload because the benefit of more slots per hash class is outweighed by the cost of increased hash-collision density: fewer buckets means more competing keys per bucket, and the fixed 3-swap promotion budget cannot rescue items stranded deep in a doubled-width bucket. The cycle 0001 hit-rate regression from wider buckets was not an artifact of linear-scan latency — it reflects a genuine capacity-partition mismatch with the trace's collision structure, and the 3-swap policy does not compensate for it.
