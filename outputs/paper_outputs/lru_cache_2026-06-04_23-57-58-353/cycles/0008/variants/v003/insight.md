# Variant v003 Insight

## Helped

## Hurt

- hit-rate: replacing the cumulative `freq` counter with a two-point IRR timestamp dropped hit rate, because IRR only captures the gap between the two most recent accesses and discards the long-term access history that freq preserves across demotion cycles via halving.

## Takeaway

Transplanting IRR eviction into the ghost-admission framework did not produce complementary gains on workload-a. The freq-based parent retains a decay-weighted signal (halving on demotion) that reflects broad access history; the IRR snapshot is a single inter-reference gap that is sensitive to recency noise and treats a node with two tightly-spaced accesses in the distant past as equally hot as one accessed twice in rapid succession just now. For this workload, cumulative frequency is a stronger predictor of future reuse than temporal clustering between the two most recent accesses, so the loss of historical signal outweighs any benefit from detecting burst patterns.
