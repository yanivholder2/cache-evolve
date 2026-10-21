# Variant v003 Insight

## Helped

- hit-rate: Per the hypothesis, a second "protected" bit splits entries into never-reused (probation) and once-reused (protected). Probation entries are evicted first, so one-touch scan traffic no longer displaces keys that have proven reuse. Hit-rate on workload-a rose modestly over the parent's single referenced bit. This is one trace, so the gain is small and not shown to generalize.

## Hurt

- None measured. hit-rate is the only reported dimension. The protected bit lives in spare metadata bits, so memory should not grow. The multi-pass victim scan, with its demotion fallback, could cost CPU time that no dimension captures. I could not read the overlay sources, so this rests on the hypothesis text and the two fitness vectors.

## Takeaway

Splitting the reuse signal into two tiers (SLRU-style) gave a small gain over the one-bit reference flag, with no counter and no extra allocation. Most of the benefit of reuse-aware eviction seems to have been captured by the first bit. The second tier adds only a little, so its extra scan complexity needs to be weighed against that.
