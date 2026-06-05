# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Capping frequency counters at 32 has no measurable effect on hit-rate for workload-a. The hypothesis predicted this would improve hit-rate by preventing ancient hotspots from monopolizing the frequency ranking, but the periodic decay mechanism (right-shift every `capacity * 16` operations) already handles the dynamic rebalancing needed. Either this workload doesn't generate frequency values near saturation, or the existing decay threshold is sufficient to refresh priorities fast enough that an arbitrary cap at 32 adds no practical benefit. The change is neutral: it doesn't hurt, but the theoretical advantage doesn't materialize on this trace.
