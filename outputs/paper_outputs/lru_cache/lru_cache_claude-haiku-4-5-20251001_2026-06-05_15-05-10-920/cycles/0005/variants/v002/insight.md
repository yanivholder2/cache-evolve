# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Multiplying by 0.7 instead of 0.75 decays frequencies too aggressively, causing recently accessed items to lose their frequency advantage faster than is optimal for this workload; items that should remain distinguished lose relative priority during reset cycles.

## Takeaway

More aggressive frequency decay (0.7× vs 0.75×) paradoxically worsens hit-rate. While the hypothesis anticipated faster adaptation to phase shifts, the observed result suggests that discarding frequency history too quickly undermines the cache's ability to distinguish between frequently and infrequently accessed items. The parent's gentler 0.75× coefficient better balances recency and frequency on this workload, indicating diminishing returns beyond the already-tuned value.
