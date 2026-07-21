# Variant v001 Insight

## Helped

- **hit-rate**: Raising the CLOCK_BIT threshold from freq≥1 to freq≥3 restores cold/warm discrimination in target buckets — items accessed only once or twice remain unprotected and are evicted by the LFU-minimum scan on their own merits, while only genuinely high-reuse items (three or more hits) earn clock protection, shrinking the uniformly-warm stalemate that had neutered the parent's clock.

## Hurt

- **per-op-time**: Put tail latency (p50 and p99) increased substantially relative to the parent; the freq≥3 gate shifts which items accumulate CLOCK_BIT, and when high-frequency items cluster in a bucket the clock pre-pass must rotate more warm entries before settling on a cold victim, making the eviction path in `clock_evict_prepare` more expensive on the put side for hot working-set regions.

## Takeaway

Gating CLOCK_BIT on freq≥3 instead of every hit does recover a small but real hit-rate gain over the parent by giving the LFU-minimum scan genuine cold candidates to choose from, confirming the saturation hypothesis. The trade-off is that the same workload now concentrates clock-protected items in hot buckets, surfacing a latency cost in `clock_evict_prepare` at eviction time; future work should investigate whether a lower threshold (e.g., freq≥2) or a cap on consecutive warm rotations can preserve the hit-rate gain while containing the put-path tail overhead.
