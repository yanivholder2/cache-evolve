# Variant v002 Insight

## Helped

(none)

## Hurt

(none)

## Takeaway

Introducing a two-tier burstiness-window filter in victim selection—where items with `burstiness_window == 0` are prioritized over those with recent activity before fallback—produces no measurable change in hit-rate over the parent's simpler frequency-cohort approach. This suggests that dormancy signals derived from burstiness window resets do not refine victim selection beyond what age-normalized frequency scoring already captures within the cold cohort. The added filtering complexity lacks empirical support on this workload.
