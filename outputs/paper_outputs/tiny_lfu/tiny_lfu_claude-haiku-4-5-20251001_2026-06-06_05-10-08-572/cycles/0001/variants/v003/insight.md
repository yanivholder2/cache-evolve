# Variant v003 Insight

## Helped

- per-op-time: Relaxed admission policy reduces decision-making overhead, with get operations ~17% faster and put operations also accelerated by accepting marginal candidates more readily.

## Hurt

- hit-rate: Admitting items with equal frequency to victims rather than strictly higher frequency admits lower-quality candidates, resulting in a slight hit-rate regression on this workload.

## Takeaway

Relaxing the admission barrier from strict inequality (>) to equality (>=) improves operational latency at the cost of hit rate. This suggests that cache performance involves a fundamental trade-off: broader admission criteria streamline operations but dilute the cache with marginal items. On workload-a, the strict frequency threshold better preserves high-value resident set composition despite the latency cost. Admission policy tuning must balance throughput against hit quality rather than assuming relaxed criteria universally improve cache behavior.
