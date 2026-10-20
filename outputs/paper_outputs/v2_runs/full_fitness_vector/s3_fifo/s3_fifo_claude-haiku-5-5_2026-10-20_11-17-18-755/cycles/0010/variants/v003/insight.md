# Variant v003 Insight

## Helped

- hit-rate: Hit-rate rose slightly over the parent despite the bit array's false positives, so the ghost-hit admission rule still works when ghost membership is approximate and epoch-cleared rather than exact per key.
- per-op-time: Put-path latency dropped sharply at both p50 and p99, consistent with removing per-key expiry bookkeeping and the ghost key hash set from the insert path, as the hypothesis mechanism describes.
- per-op-time: Get-path latency also dropped at both p50 and p99, which the ghost-only change does not directly explain; this is plausibly a side effect of the smaller container layout or measurement variance and should be rechecked before attributing it to the change.

## Hurt

- metadata-size: The S3-FIFO struct grew, so the container-level footprint went up rather than down; the bit array lives on the heap and is not visible to this probe, so the claimed ghost storage saving is not confirmed by the measured dimension.

## Takeaway

Replacing exact ghost keys with an epoch-cleared two-hash bit array mainly buys put-path speed, and the hit-rate cost of false positives was small enough not to show up as a regression. The change does not reduce measured container size, so its footprint benefit depends on heap accounting that this fitness vector does not capture.
