# Variant v003 Insight

## Helped

- (none): no measured dimension improved.

## Hurt

- hit-rate: Setting `freq = 1` on ghost-hit entries gives them an extra second-chance pass in `evict_main`, so they stay in main longer and crowd out entries with real in-main reuse, which lowers hit-rate slightly versus the baseline.
- metadata-size / per-op-time: the fitness vector for this variant has no entries for these dimensions, so nothing can be said about them. The change adds no fields and one assignment, so large shifts are unlikely.

## Takeaway

A ghost hit shows reuse at a distance longer than the small queue. It does not show that the key will be reused again soon. Giving such keys protection from the start made main eviction retain them at the expense of other entries and gave no gain on this workload. In S3-FIFO, freq 0 on ghost promotion appears to be the better default, because a key has to earn its main-queue protection through hits after it arrives.
