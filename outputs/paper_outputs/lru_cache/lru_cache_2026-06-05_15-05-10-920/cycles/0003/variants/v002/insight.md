# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Dynamic threshold based on average frequency distribution (bounded by `max(capacity * 8, min(capacity * 32, 2 * avg_freq))`) reduces adaptability compared to the fixed `capacity * 16` interval, suggesting the overhead of tracking 512-entry frequency history and recomputing bounds on every operation outweighs any benefit from phase-responsiveness on this workload.

## Takeaway

Replacing the fixed `capacity * 16` decay interval with a dynamically-computed threshold based on observed frequency distribution hurt hit-rate. While the hypothesis assumed workload phase shifts would benefit from adaptive decay timing, the added complexity—maintaining a rolling frequency window, computing average frequencies per operation, and recalculating bounds—introduced overhead that degraded performance. The fixed interval was simpler and more effective, suggesting either this workload lacks sharp phase transitions or the gains from adapting decay timing do not justify the per-operation tracking cost.
