# Variant v003 Insight

## Helped

_(no dimensions improved)_

## Hurt

- **hit-rate**: Raising the CLOCK_BIT threshold from `freq > 0` to `freq >= 3` strips clock protection from items with frequency 1–2; on workload-a these mid-frequency items form a substantial fraction of the hot set, so the clock now evicts them as cold rather than giving them a second chance, directly lowering the hit rate compared to the parent.
- **per-op-time**: Despite replacing the O(n) LFU-minimum scan with an O(1) position-0 return, the p99 for both get and put rose; the sparse CLOCK_BIT threshold causes more items to be treated as cold and evicted early, increasing promotion churn (more `promote_to_target` → `clock_evict_prepare` calls under pressure) that inflates tail latency rather than relieving it.

## Takeaway

The two-change bundle conflates two separate bets: switching to position-0 eviction (to make the clock rotation meaningful) and raising the CLOCK_BIT threshold to freq≥3 (to reduce uniform warming). The sparsification threshold is the regressor — workload-a's hot items are concentrated at frequency 1–2, so a freq≥3 guard leaves most of the working set permanently cold, defeating the second-chance protection the clock was meant to provide. Future attempts should either test position-0 eviction alone (keeping CLOCK_BIT set on every hit) or probe a lower sparsification threshold (e.g., freq≥2) before raising it further.
