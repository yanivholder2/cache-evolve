# Variant v002 Insight

## Helped

- **per-op-time**: Adding `clock_evict_prepare` before eviction in `promote_to_probing` pre-positions a cold victim at slot 0 before the LFU minimum-scan runs, reducing the scan's traversal cost on full probing buckets and bringing down put-path tail latency substantially.

## Hurt

- **hit-rate**: The probing clock pre-pass retains warm probing residents (CLOCK_BIT set, freq≥3) for an extra round; when a target-promoted item triggers eviction from a full probing bucket, it tends to displace the cold item the clock rotated to the front rather than the warm item — but that warm item, already unable to re-enter target (vacancy gate silenced by a full target), now occupies a probing slot an additional eviction cycle longer, blocking subsequent target promotees from settling in probing and causing them to cascade before accumulating re-entry frequency.
- **metadata-size**: This variant's overlay covers only `dash_segment.rs`, so the evaluation runs against the base `item.rs` whose `Item<K,V>` struct is larger than the one the parent variant carried; the probing clock extension itself adds no new fields to `Item`.

## Takeaway

Extending the two-chance clock symmetrically to probing does pay off on the latency side — pre-rotating warm items front-loads cold-victim selection and cuts put-path tail cost — but the hit-rate falls: protecting warm probing residents from prompt demotion keeps the probing tier sticky, slowing the target→probing→target circulation that drives hit rate on this workload. The latency gain is real, but the clock's admission-blocking effect in probing is the wrong trade-off here; future work might gate the probing clock pre-pass on bucket occupancy rather than running it unconditionally.
