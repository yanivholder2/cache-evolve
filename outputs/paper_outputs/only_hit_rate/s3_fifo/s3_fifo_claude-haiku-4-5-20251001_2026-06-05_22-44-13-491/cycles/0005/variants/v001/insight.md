# Variant v001 Insight

## Helped

- **hit-rate**: Reducing the small-queue capacity further from 1/8 to 1/9 continues to tighten the admission filter, preserving more space in the main queue for high-confidence items and enabling the frequency threshold (freq > 1) to better separate genuine repeaters from noise, resulting in a marginal but measurable improvement.

## Hurt

(none)

## Takeaway

The saturation boundary has not yet been reached. Further reduction of the admission queue below 1/8 continues to yield positive returns, albeit with diminishing magnitude. The tighter 1/9 ratio allows only items with stronger reuse patterns to be promoted into the main queue, demonstrating that the workload contains sufficient signal in access patterns to benefit from more selective filtering. The admission gate can still be tightened without sacrificing working-set diversity, indicating the optimal ratio may lie even tighter than 1/9.
