# Variant v003 Insight

## Hurt

- **per-op-time**: Age-aware threshold computation (dividing queue length by small_size on every eviction) adds branch overhead and floating-point arithmetic, increasing get p50 by 1 ns and put latencies by 35–244 ns, offsetting any theoretical benefits.

## Helped

(No dimensions improved)

## Takeaway

The age-aware promotion threshold—loosening the frequency bar to 1 for items in the back 30% of the small queue—failed to improve hit-rate despite targeting the frequency-starvation pattern. The mechanism did not unlock additional promotions in this workload, suggesting either the back-30% window is too narrow, items that would benefit are already reaching freq=2, or the workload's access pattern doesn't create the starvation scenario the hypothesis aimed to fix. The added overhead (age_ratio computation and conditional threshold selection) pushes latency slightly higher, making this variant both less accurate and less efficient than v001.
