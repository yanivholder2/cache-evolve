# Variant v003 Insight

## Hurt

- **hit-rate**: Eliminating recency information from victim selection removes a discriminating signal that, despite potential noise in age tracking, still meaningfully contributes to correct eviction decisions; the 0.31% decrease shows that even with improved frequency estimates, age alone better captures working-set membership than frequency alone.

## Takeaway

Frequency precision alone is insufficient to replace the recency signal in eviction decisions on this workload. While Cycles 0001-0002 improved sketch accuracy and the hypothesis reasoned that frequency estimates alone might then suffice, the 0.31% hit-rate loss demonstrates that temporal locality — which age captures directly — remains essential. The hypothesis's assumption that "frequency is sufficient if estimates are precise" does not hold; recency adds orthogonal information about access patterns that frequency cannot substitute for, even under optimized sketch parameters.
