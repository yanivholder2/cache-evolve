# Variant v002 Insight

## Hurt

- **hit-rate**: Dual-signal re-admission strategy produced less accurate re-access predictions than the baseline single-signal approach, likely because the boosting formula (combining recency and frequency equally) was too conservative in determining which ghost hits should receive high initial frequency.

## Helped

- **per-op-time**: Both `get` and `put` operations became significantly faster, suggesting the revised eviction and ghost queue operations are more efficient to compute than the baseline implementation.

## Takeaway

The dual-signal re-admission strategy failed to improve cache hit-rate despite adding frequency decay tracking and position-based recency scoring. Equal weighting of the two signals and the conservative boost formula (range 2–4) appears to have disadvantaged re-admitted items; they start with insufficient frequency to survive main queue eviction. The efficiency gains (sub-microsecond latency improvements) came from streamlined eviction logic, not better prediction, revealing a disconnect between computational simplicity and access pattern accuracy.
