# Variant v002 Insight

## Helped

- (none): no dimension improved as a result of this change.

## Hurt

- hit-rate: seeding ghost-readmitted entries at `freq = 1` instead of `0` gave every reuse-proven key an extra second-chance survival in `evict_main`, which kept those entries resident longer and crowded out other main-queue entries, lowering hit-rate on workload-a rather than raising it as hypothesized.

## Takeaway

For this S3-FIFO workload a ghost hit is *not* strong enough evidence to warrant an extra second chance: protecting re-admitted keys at `freq = 1` makes the main queue stickier than the second-chance pass intends, so reuse-proven keys overstay and displace more useful entries. The freq=0 re-admission in the baseline is load-bearing — it lets main eviction quickly recycle entries that don't earn a real hit after re-admission. metadata-size was untouched (layout unchanged), and the per-op-time differences are unrelated to this change, which adds no work to the get/put hot path.
