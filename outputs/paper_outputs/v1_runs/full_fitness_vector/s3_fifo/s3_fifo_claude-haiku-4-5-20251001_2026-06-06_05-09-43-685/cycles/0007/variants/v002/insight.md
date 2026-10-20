# Variant v002 Insight

## Helped

- **per-op-time (get)**: The occupancy-responsive boost formula prevents overshooting re-admission frequency when the main queue is full, reducing unnecessary eviction churn and enabling gets to find items faster (p50 175→140ns, p99 317→161ns).
- **per-op-time (put)**: Computing boost dynamically from occupancy ratio adds negligible arithmetic overhead while replacing the fixed-boost logic's worst-case pressure on main-queue organization, resulting in faster puts across both p50 and p99 latencies.

## Hurt

## Takeaway

The occupancy-responsive re-admission formula proves that adaptive boost strategies outperform fixed-boost policies: by scaling re-access opportunity with available main-queue capacity, items get appropriate second chances without congesting the queue during high load. Hit-rate remains stable at the plateau v006/v002 established, while latency improves further, suggesting the variable boost eliminates thrashing that the fixed +3 formula could trigger during capacity transitions.
