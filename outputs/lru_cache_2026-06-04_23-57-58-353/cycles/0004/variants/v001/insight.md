# Variant v001 Insight

## Helped

- hit-rate: Widening the scan from the LRU quarter to the LRU third gave the frequency selector a modestly larger candidate pool, producing a marginal improvement over the parent.

## Hurt

## Takeaway

Expanding the demotion scan from 25% to 33% of the protected segment moves hit-rate by only a tiny amount, suggesting the 1/4 fraction already captures most of the useful frequency variation near the LRU end of the segment. The extra candidates exposed by the wider window are not meaningfully lower-frequency than those already visible at 1/4, so the larger scan does little more work than the narrower one. Fraction tuning above 1/4 appears to offer diminishing returns for this workload.
