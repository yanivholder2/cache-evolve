# Variant v001 Insight

## Helped

(None measured in this evaluation)

## Hurt

- **hit-rate**: Pushing the reduction factor from 0.71 to 0.75 overshoots the geometric optimum, resulting in buckets that are too wide relative to the workload's actual collision patterns, degrading cache effectiveness.

## Takeaway

The 0.75 reduction factor is too aggressive for this workload. While wider buckets theoretically reduce collision clustering, the 0.71 factor from the parent already positioned the geometry near its optimal point. Increasing aggressiveness to 0.75 suggests that collision reduction has already hit saturation—further geometry expansion wastes capacity on overly-wide buckets that don't compress the eviction stream any further, and may introduce other trade-offs that outweigh collision benefits. The geometric design space has a well-defined optimum somewhere between 0.71 and 0.75, and this variant confirms that the high side of that range overshoots the target.
