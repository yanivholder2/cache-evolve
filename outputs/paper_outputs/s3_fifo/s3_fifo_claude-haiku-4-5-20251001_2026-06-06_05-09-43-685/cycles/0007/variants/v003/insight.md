# Variant v003 Insight

## Helped

- **per-op-time**: Recency-based ghost re-admission computation (position in queue divided by length) is significantly cheaper than the baseline's ghost handling, reducing get latency by 43% and put latency by 57%.

## Hurt

- **hit-rate**: Recency signal alone is less predictive than the baseline's approach; the boost scaling (3–4 for recent items, 2 for aged) didn't capture workload re-access patterns effectively, dropping hit-rate by 0.7%.

## Takeaway

Recency position in the ghost queue is a computationally efficient re-admission signal that dramatically improves operation latency, but sacrifices hit-rate fidelity. The baseline's original ghost handling (likely frequency or occupancy-based) better captures which evicted items will actually be re-accessed in this workload, suggesting that temporal distance from eviction is less predictive than the original signal.
