# Variant v003 Insight

## Helped

- hit-rate: Replacing plain LRU with a segmented LRU (probation plus protected) and a count-min-sketch admission gate keeps one-hit-wonder scans out of the protected segment. Counting rejected misses lets a key that keeps coming back build frequency and eventually get admitted. Together these beat the recency-only baseline on the LIRS workload.

## Hurt

- metadata-size: The fitness vector has no metadata-size entry for this variant, so I can't confirm the claimed saving. The change adds a 4-row u8 sketch plus segment bookkeeping to a baseline that only had the node list and map. The no-doorkeeper simplification is the only part that trims metadata, and it is relative to a variant I can't see.
- per-op-time: The vector has no per-op-time entry either, so the claimed speedup is unverified. The sketch hashing in `put()` and on `get()` misses, plus the promotion and demotion between segments on every hit, adds work the baseline's single list move did not have. Skipping sketch updates on resident hits only partly offsets that.

## Takeaway

Frequency-aware admission with an SLRU layout is what raised hit-rate; the sketch and segments are paid for in per-op and metadata cost. The evaluation only recorded hit-rate, so the hypothesis's speed and size claims were never validated. A later variant needs the per-op and metadata measurements before the sketch-skipping optimization can be judged.
