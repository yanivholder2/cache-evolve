# Variant v003 Insight

## Helped

- **hit-rate**: Reducing the small-queue admission threshold from 1/6 to 1/8 of capacity continues to improve selectivity, allowing only items with the strongest reuse signals to pass through the admission gate and occupy prime main-queue space, resulting in denser concentration of frequently-accessed items.

## Hurt

(none)

## Takeaway

The continued improvement from v001 to v003 confirms that the workload is still below the utilization saturation cliff. Tightening the admission gate from 1/6 to 1/8 maintains the gains trajectory, indicating the optimal small-queue ratio lies somewhere tighter than 1/6. The aggressive filtering does not yet constrain working-set diversity—items demonstrating genuine reuse patterns still graduate successfully to the main queue, and the main queue remains effective at retaining high-confidence repeaters. This variant demonstrates that S3-FIFO's efficiency improves through deliberate under-provisioning of the admission queue, as long as the filtering logic (frequency > 1 for promotion) accurately separates signal from noise.
