# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Inserting freq=2 promotions at the main-queue midpoint instead of the front had zero effect on hit-rate for this workload. The change only affects items with exactly `freq == 2` (those with `freq >= 3` still land at the front, identical to the parent). Two likely explanations: the workload-a trace produces very few freq=2 promotions from small relative to total traffic, so the modified path is rarely exercised; or eviction pressure in main advances quickly enough that the positional gap between front and midpoint collapses before the next access cycle, making the initial insertion point irrelevant to survival. The result confirms that fine-grained placement within main does not move the needle on this workload — the real driver of hit-rate is the promotion threshold and frequency ceiling, not where within main a promoted item lands.
