# Variant v003 Insight

## Helped

## Hurt

## Takeaway

H03 pre-sized the key index at construction, a change meant to cut per-op-time by avoiding rehashing, while leaving eviction untouched. Hit-rate matched the parent exactly, which fits a change that does not alter eviction order. The variant's fitness vector reports only hit-rate, though, so this variant offers no evidence on whether pre-sizing helped per-op-time or metadata-size, and the hypothesis's central claim remains unverified.
