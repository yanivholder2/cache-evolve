# Cross-Cycle Insights

## Recurring wins

- **Frequency-based protection of hot items**: Consistent winner across cycles (0001 v001, 0002 v003). Simple frequency counters, even without adaptation, unlock ~8–9% hit-rate gains by protecting reuse-prone items from early eviction. The core insight—recognizing access-skewed workloads where some items matter far more than others—remains valid.

- **Fixed, simple strategies beat adaptive complexity**: Periodic frequency decay with fixed `capacity × 16` interval (0002 v003) outperformed adaptive variants (0003 v002) and zone-based hybrids. The workload does not appear to reward real-time adaptation; a static, lightweight mechanism captures the necessary behavior.

- **Temporal adaptation works when decoupled from partitioning**: Periodic frequency decay successfully enables the cache to shed ancient hotspots and weight newly-popular items without requiring zone boundaries or frequency-distribution tracking. The mechanism is low-overhead and generalizable.

## Recurring failures

- **Zone-based partitioning consistently regresses**: Introduced in 0001 v002, refined in 0002 v002, and expanded in 0003 v001 and v003—every variant hurt hit-rate. Artificial promotion barriers between zones mask frequency signals and evict moderately-frequent items before they prove reuse value. Unified frequency competition across the entire cache is superior.

- **Adaptive and hybrid mechanisms add overhead without return**: Adaptive decay thresholds (0003 v002), zone-hybrid frequency+LRU logic (0002 v002), and independent zone decay rates (0003 v001) all either regressed or failed to beat the parent. The workload either lacks the structural properties (phase transitions, bimodality) that justify adaptation, or the cost of tracking and computation outweighs marginal phase-shift gains.

- **Conservative re-insertion and insertion-position tuning don't yield gains**: Ghost-list second-chance (0001 v003) and tie-breaking direction reversal (0002 v001) both degraded performance. Re-insertion policy appears orthogonal to the primary cache-improvement driver (frequency protection).

## Open questions

- **Can within-frequency refinements move the needle?** Counter saturation behavior, alternative decay patterns (exponential vs. linear right-shift), and tie-breaking rules when evicting equal-frequency items remain unexplored. These avoid partitioning overhead while probing remaining margin.

- **How portable is the `capacity × 16` decay interval?** It generalized from 0002 to 0003 without tuning, but cross-trace and cross-scale portability is unknown. Does the interval scale with cache size, workload skew, or other parameters?

- **Is there a lower-cost adaptation signal than frequency statistics?** Hit-miss ratio trending, eviction-frequency spikes, or other online signals might capture phase shifts more efficiently than distribution-aware logic, if phase shifts are real.

- **Are we near a fundamental limit on this workload?** The fact that all three orthogonal refinements in 0003 regressed suggests the parent may be at or near a local optimum. Further gains may require qualitatively different insights (e.g., inline prefetching, request coalescing) rather than eviction-policy tweaks.

## Per-policy notes

- **Frequency-based eviction**: Dominant across all cycles when paired with periodic decay. Simple global counters, even without distribution awareness, capture the essential insight. Zone modifications consistently backfire; hybrid frequency+LRU logic adds complexity without corresponding gain.

- **Zone-based partitioning**: Consistently underperforms across all contexts (0001 v002 vs. frequency winner by 0.7%, 0002 v002 and v003 regressed, 0003 v001 and v003 regressed). The workload benefits from unified frequency competition, not segregation.

- **Ghost-list / second-chance**: Concept failed (0001 v003). Re-insertion at the middle of LRU order was too conservative, and coupling with frequency tracking was not explored. Unlikely to be salvageable without fundamental redesign.

- **Decay interval and counter saturation**: Fixed `capacity × 16` interval is robust and simple. Counter saturation behavior (when do frequencies max out?) and alternative decay patterns (exponential, logarithmic) remain unexplored avenues for refinement.
