# Variant v001 Insight

## Helped

- **per-op-time (put)**: Marginally reduced by 1 ns, though the direction of change does not align with the stated hypothesis of reducing overall per-op latency through more permissive promotion.

## Hurt

- **per-op-time (get)**: Increased by 1 ns relative to parent, moving away from the latency target despite the hypothesis expecting improvements.
- **per-op-time (overall)**: The summary metric degraded from 79 ns to 80 ns, contradicting the hypothesis's prediction that reducing promotion selectivity would improve latency.

## Takeaway

The attempt to reduce per-op-time by increasing promotion permissiveness (divisor 4→3) failed to improve latency and slightly degraded the get operation. On the 75/25 geometry with 8 stash buckets, integer division means the promotion threshold stayed at 2 in both variants, suggesting the hypothesis's quantitative assumption did not hold. More fundamentally, the hypothesis assumed latency was bottlenecked by aggressive promotion filtering that forces hot items to be repeatedly probed in the target tier; the null-or-negative result suggests either (a) the latency bottleneck lies elsewhere in the lookup or eviction path, or (b) the 75/25 allocation is already near-optimal and further tuning in this direction is counterproductive. The inverse-coupling principle observed in earlier cycles may have limited applicability at this allocation point.
