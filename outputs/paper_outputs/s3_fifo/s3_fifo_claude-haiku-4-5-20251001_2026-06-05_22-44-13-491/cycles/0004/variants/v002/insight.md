# Variant v002 Insight

## Helped

- **hit-rate**: Increasing the maximum frequency depth from 3 to 4 grants items in the main queue an additional eviction round before permanent discard, better protecting moderately-reused items that v001's strict 1/6 small-queue admission gate allowed through.

## Hurt

(none)

## Takeaway

The combination of v001's 1/6 small-queue structural selectivity with MAX_FREQ=4 produces complementary effects: the tighter admission gate concentrates the main queue on higher-confidence items, while the deeper frequency budget ensures those moderately-reused items survive longer under eviction pressure. The modest improvement confirms the two optimizations are genuinely orthogonal and reinforce each other on this workload.
