# Variant v001 Insight

## Helped

(none)

## Hurt

- hit-rate: Relaxing the admission gate from strict `>` to `>=` allows items with equal frequency estimates to the victim to be admitted, reducing selectivity and causing more low-utility items to displace cache residents.

## Takeaway

Relaxing the admission gate threshold from `incoming_freq > victim_freq` to `incoming_freq >= victim_freq` caused hit-rate to decline slightly (0.7429 → 0.7414). This confirms that the gate's strict selectivity remains optimal even with the refined victim-selection policy (`age / (frequency + 2.0)`). Items with equal frequency to the victim should remain rejected; the stricter gate enforces a meaningful distinction that the improved eviction logic alone cannot compensate for.
