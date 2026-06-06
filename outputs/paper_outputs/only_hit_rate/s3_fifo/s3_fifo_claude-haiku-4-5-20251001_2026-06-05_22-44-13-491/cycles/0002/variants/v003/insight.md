# Variant v003 Insight

## Helped

- **hit-rate**: Reducing the small queue admission size from 1/4 to 1/5 of capacity enforces stricter frequency-based filtering, allowing only higher-reuse items to graduate to the main queue, which improves overall hit rate through better item selection.

## Hurt

(none)

## Takeaway

Narrowing the small-queue admission gate by reducing its size from 1/4 to 1/5 of capacity tightens the frequency selectivity threshold, allowing only items that demonstrate stronger reuse patterns to reach the main queue. This structural constraint improves cache hit rate by raising the bar for promotion, ensuring the main queue retains items with more reliable frequency evidence. The result validates that frequency-based filtering benefits from constrained opportunity rather than broad admittance.
