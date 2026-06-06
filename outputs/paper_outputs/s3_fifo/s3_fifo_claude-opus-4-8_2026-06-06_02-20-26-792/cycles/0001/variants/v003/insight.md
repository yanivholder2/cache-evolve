# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Bounding the ghost queue against `ghost_size = capacity` instead of `main_size` keeps stale evicted keys in history far longer, and every late ghost hit re-admits straight into main via `push_front` after `evict_main()` — so cold keys reaccessed after a long gap now skip the small queue and displace genuinely hot main residents, polluting the resident set.

## Takeaway

Widening the reuse-detection window is not free on this LIRS workload: a ghost queue sized to full capacity converts long-gap reaccesses into direct main re-admissions that crowd out hotter entries, so deeper history slightly hurt hit-rate rather than helping. History depth and the direct-to-main re-admission rule are coupled — deepening one without gating the other backfires.
