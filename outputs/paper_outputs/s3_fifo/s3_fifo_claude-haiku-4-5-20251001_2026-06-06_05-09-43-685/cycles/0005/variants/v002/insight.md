# Variant v002 Insight

## Helped

- **per-op-time (get)**: Removing the growth adjustment at 0.5 occupancy threshold eliminates a frequent resize operation that v003 performed on every put during low-load phases, reducing get p50 by 43ns despite the shared occupancy-check path.
- **per-op-time (put)**: Reducing the adjustment frequency by removing the 0.5 threshold and tightening bounds (15%–20% vs 10%–25%) cuts resizing overhead and the associated eviction work, improving put p50 by 64ns.

## Hurt

- **hit-rate**: Removing the occupancy-responsive growth at low load (≤0.5) prevents the small queue from expanding to capture more reachable items, losing 0.26% hit-rate as fewer items reach the freq > 2 graduation threshold before eviction.

## Takeaway

The latency overhead in v003's occupancy-aware sizing was primarily driven by the *frequency* of adjustments (growth checks on every put), not just the magnitude of the bounds. By removing the low-occupancy growth adjustment, v002 recovers most of the latency cost (get and put both improve notably) but forfeits the hit-rate gain, revealing that the two benefits are coupled—adaptive growth is costly per-operation, but necessary for the hit-rate improvement. This demonstrates that structural tuning in cache eviction requires accepting latency–efficiency trade-offs; there is no "free" adaptation without per-operation overhead.
