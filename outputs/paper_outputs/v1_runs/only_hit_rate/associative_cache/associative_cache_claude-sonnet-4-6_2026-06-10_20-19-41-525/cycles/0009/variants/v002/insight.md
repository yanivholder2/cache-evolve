# Variant v002 Insight

## Helped

- hit-rate: Introducing the 4× wide-bucket geometry overlay (32 buckets × 64 slots instead of 128 × 16) gave each SLRU bucket four times as many probationary and protected slots, enlarging the eviction pool enough to retain more of the genuine working set across both segments.

## Hurt

## Takeaway

Wide-bucket geometry does provide a measurable lift to SLRU, confirming that the narrow per-bucket capacity inherited from the baseline was suppressing hit rate rather than the tier-separation logic itself being the sole bottleneck. However, SLRU with the same wide geometry that powers the single-queue winners still falls well short of those variants, which means the two-queue demotion/promotion overhead or its eviction priority (probationary-first regardless of frequency) is itself the limiting factor — not geometry alone. Further gains on this branch require revisiting the SLRU eviction and promotion rules rather than the bucket layout.
