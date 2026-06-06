# Variant v002 Insight

## Hurt

- hit-rate: Frequency-tier-based victim selection slightly underperforms the baseline approach, suggesting that the discretization of frequency ranks into fixed percentile tiers loses discriminative power compared to continuous scoring.

## Takeaway

H02 hypothesized that replacing continuous victim-scoring with frequency tiers would improve robustness by grouping items into frequency cohorts and always preferring the coldest tier. In practice, the tier-based approach performed marginally worse. This teaches that the baseline's simpler victim selection—picking the oldest item when incoming frequency exceeds victim frequency—already balances frequency and recency without the overhead of percentile computation. The hypothesis conflated "simpler = more robust" with "worse performance", but the lack of fine-grained frequency discrimination within each tier appears to leave better candidates in the cache longer.
