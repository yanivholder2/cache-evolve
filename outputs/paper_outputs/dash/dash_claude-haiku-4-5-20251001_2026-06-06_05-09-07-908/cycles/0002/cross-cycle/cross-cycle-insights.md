# Cross-Cycle Insights

## Recurring wins

- **Hot-path efficiency improvements consistently improve both latency and hit-rate** on Zipfian workloads; all six winners across cycles 0001–0002 focused on eliminating unnecessary work rather than complex restructuring.
- **Geometry allocation (hash-indexed vs stash capacity ratio) is the primary tuning lever**, showing sustained improvement across every allocation profile tested (25/75 baseline → 50/50 → 62.5/37.5).
- **Simple, low-cost filtering mechanisms eliminate false-positive transitions**: Frequency-gated promotion (counter ≥ 2 threshold) improves both metrics by reducing cache churn and cascading demotions.
- **Orthogonal optimization axes operate without design-level conflicts**: Promotion logic, bucket geometry, and struct packing each address independent bottlenecks.

## Recurring failures

- **Mechanically independent optimizations can interact negatively in practice**: Combining back-insert promotion with 50/50 geometry regressed latency despite hit-rate gains, revealing that implementation-level feedback loops matter even when logical dimensions are distinct.
- **Composition requires careful sequencing**: Not all combinations of separately-winning variants improve performance when applied together.

## Open questions

- Do aggressive geometry (62.5/37.5) and frequency-gated promotion compose without regression, or will latency trade-offs emerge again?
- Is there a saturation point for geometry concentration, or does further hash-indexed bias continue to improve performance?
- Can promotion frequency thresholds adapt per-workload, or is the counter ≥ 2 threshold near-universal across Zipfian distributions?
- How do these hot-path optimizations perform on non-Zipfian distributions (uniform, bimodal, or multi-modal workloads)?

## Per-policy notes

- **Promotion ordering**: Back-insert alone improves hit-rate; frequency-gated filtering of false-positive promotions is cost-free in isolation and improves both metrics.
- **Bucket geometry**: The primary performance lever; every allocation shift (25/75 → 50/50 → 62.5/37.5) yielded wins, with hash-indexed capacity consistently more valuable than stash on Zipfian workloads. No saturation observed yet.
- **Memory layout**: Item struct compaction achieved gains via improved cache efficiency with minimal complexity and no downside.
- **Composition effects**: Latency can regress when combining improvements despite hit-rate gains; test combinations carefully rather than assuming orthogonality guarantees performance additivty.
