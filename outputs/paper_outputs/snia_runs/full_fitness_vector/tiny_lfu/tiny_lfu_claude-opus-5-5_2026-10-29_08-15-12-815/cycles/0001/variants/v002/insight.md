# Variant v002 Insight

## Helped

- hit-rate: Halving every counter after `10 × capacity` effective increments, with counters capped at 15, means keys that were hot early in the trace lose their admission advantage. Newer working-set blocks now win the strict `>` contest against stale LRU victims. Hit-rate rose in all four trace×size cells. The biggest gains were at the small size (16384) on both web_0 and hm_0. That fits the mechanism: a small cache runs admission contests far more often, so pinned stale data cost the most there.
- per-op-time: The `put` tail got better while the median did not. A plausible cause is the early return in `increment` once the row minimum hits `MAX_COUNT`, which skips counter writes for saturated hot keys. Treat this as tentative, since the `reset` sweep should make tails worse, not better.

## Hurt

- per-op-time: Median `get` and `put` latency both went up. Each `increment` now takes two passes over the rows (find the minimum, then do a conditional bump) instead of one blind bump. It also pays, amortized, for `reset`, which sweeps all `depth × width` counters every `sample_size` additions. Every `get` calls `increment`, so `get` carries this cost too.
- metadata-size: The headline per-node bytes did not change, and the sketch heap is the same size because storage is still `u32`. The `TinyLfuCache` struct did grow, because the sketch gained two `usize` fields (`additions`, `sample_size`).

## Takeaway

The baseline's sketch never forgets, and that was costing real hit-rate on MSR. Caffeine-style periodic halving with a 4-bit-range cap recovers it, with almost all the benefit at small capacities, where admission contests are frequent. The price is a small, steady per-operation cost from conservative update plus the amortized reset sweep. The counters now never exceed 15 but are still stored as `u32`, so packing them into 4-bit cells is a free next step that could shrink the sketch heap and the sweep cost.
