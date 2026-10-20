# Variant v003 Insight

## Helped

## Hurt

## Takeaway

H03 swapped the key index's default SipHash hasher for an inline FxHash-style hasher while leaving eviction and recency logic untouched, so hit-rate was expected to hold at the parent's value and per-op time was the dimension it targeted. Hit-rate did hold, as expected for a change that only moves bucket placement. This variant's fitness vector reports no per-op-time measurement, so the one dimension the change was meant to improve is unresolved: whether the hasher swap is a per-op win remains unknown until per-op time is recorded for this variant.
