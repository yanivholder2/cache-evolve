# Variant v002 Insight

## Helped

- hit-rate: Deepening frequency protection from MAX_FREQ=3 to MAX_FREQ=5 allows items that pass the 1/12 admission gate to survive two additional eviction rounds, extending their tenure in the main queue and capturing more reuses before permanent discard.

## Hurt

*(none)*

## Takeaway

The pairing of 1/12 structural selectivity with MAX_FREQ=5 confirms that frequency-depth and admission tightness do compound synergistically, yet the gain is modest. This suggests that at the extreme 1/12 selectivity boundary, the small queue filters so aggressively that only highest-confidence items reach the main queue, leaving less room for frequency protection to amplify further—the admission gate, not the main-queue lifespan, has become the primary bottleneck for this workload.
