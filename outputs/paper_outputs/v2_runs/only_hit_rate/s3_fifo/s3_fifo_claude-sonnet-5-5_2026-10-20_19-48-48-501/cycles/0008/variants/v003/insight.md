# Variant v003 Insight

## Helped

- hit-rate: Moving an entry from the small queue to main as soon as its freq reaches 2 on the hit path, combined with the smaller small queue and shorter ghost, keeps reused items out of small-queue churn. It improved on the parent's hit rate on the single measured trace.

## Hurt

- per-op-time, metadata-size: These dimensions were not reported for this variant, so their effect is unknown. Hit-path promotion should add work to `get` (queue removal, size-counter updates, an insertion into main). That cost was never measured.

## Takeaway

Acting on the hit path is a real lever, unlike the earlier eviction-time promotion-gate tweaks that tied with the parent. The gain came bundled with the sizing change, so the two effects can't be separated from this result. Any claim about the cost of promoting on hit needs the per-op-time dimension measured.
