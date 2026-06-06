# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Replacing the simple LRU-head demotion in `maybe_demote_from_protected()` with a k=4 minimum-frequency scan produced a marginal hit-rate regression, suggesting the scan occasionally displaces a non-LRU-head item whose removal is slightly worse than the pure recency choice.

## Takeaway

The frequency-scan demotion from the protected segment provided no benefit on top of the PROMO_THRESHOLD=6 baseline: the protected segment's LRU ordering already approximates frequency ordering well enough that a k=4 window scan finds no meaningfully colder candidate than the LRU head. The hypothesis that PROMO_THRESHOLD=6 would widen frequency heterogeneity within protected sufficiently to reward the scan was not borne out on workload-a — the frequency spread among the 4 least-recently-used protected residents is too narrow to differentiate, and picking the min-frequency non-head item can leave colder-by-recency items stranded in protected longer than the pure LRU policy would.
