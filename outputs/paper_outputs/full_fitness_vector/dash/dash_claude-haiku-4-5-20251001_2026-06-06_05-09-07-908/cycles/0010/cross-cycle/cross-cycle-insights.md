# Cross-Cycle Insights

## Recurring wins

- **Geometry allocation tuning (normal-to-stash bucket ratio)**: The most reliable and composable lever across all 10 cycles. Progressive refinement from baseline (25/75) through increasingly aggressive allocations yielded sustained improvements in both hit-rate and latency, with the sweet-spot converging to the 75/25–80/20 band (cycles 0001–0009). Geometry adjustments target the fundamental capacity bottleneck for Zipfian workloads.

- **Adaptive parameter scaling tied to geometry**: Shifting from hardcoded constants (e.g., promotion threshold=2 on all geometries) to formula-based scaling (e.g., divisor derived from stash bucket count) resolved parameter coupling and enabled reliable composition. The inverse relationship—stricter gating on tight allocations, looser gating on spacious ones—is now empirically grounded and mechanically robust (cycles 0005–0008).

- **Latency-hit-rate decoupling via new mechanisms**: Cycles 0009–0010 demonstrated that implementation efficiency (in-place counter optimization) and new tier-crossing heuristics (frequency-aware demotion gating) can improve latency substantially without sacrificing hit-rate, separating concerns that earlier cycles treated as coupled trade-offs.

- **Frequency-aware demotion cascade gating (cycle 0010)**: The first major breakthrough after geometry and promotion optimization saturated. By preventing cold items from entering target tier via frequency thresholds, this mechanism delivered ~57% get-latency improvement with minimal hit-rate trade-off (0.14% loss). It opens a new optimization axis targeting tier-crossing efficiency rather than eviction ranking.

## Recurring failures

- **Back-insert promotion**: Tested in cycles 0001, 0002, and 0004, consistently regressed latency on both get and put operations despite pairing with increasingly generous capacity allocations (50/50, 62.5/37.5, 85/15, 90/10). The MRU-insertion mechanism carries a structural cost that no amount of primary-tier capacity can overcome.

- **Hardcoded promotion thresholds across heterogeneous geometries**: When a fixed threshold (e.g., counter ≥ 2) was applied to multiple allocations, it achieved selectivity goals on some geometries but cascaded into failures on others (cycles 0006–0007). Parameter misalignment triggers eviction churn and hit-rate dilution; thresholds must adapt to capacity.

- **Pushing geometry to extremes**: Aggressive allocations beyond 85/15 (tested at 90/10 in cycle 0005) regressed hit-rate as stash capacity became insufficient for the Zipfian workload's tail. The frontier peaks around 80/20; further concentration optimizes for latency at the expense of cache effectiveness.

- **Frequency-aware eviction candidate ranking**: Cycles 0009–0010 confirmed that frequency information does not improve eviction candidate selection. On this workload, recency and frequency are well-correlated, making pure LRU sufficient; composite frequency-recency scoring adds overhead without benefit.

## Open questions

- Can frequency-aware demotion cascade gating from cycle 0010 compose with proven geometry-divisor pairs (e.g., 75/25 with adaptive divisor scaling) to compound latency improvements without hit-rate regression?

- Are there other orthogonal tier-crossing heuristics (adaptive cascade thresholds, frequency-aware promotion cascading, demotion-routing refinements) beyond the initial demotion-gating mechanism that could unlock further gains?

- Does the divisor meta-formula itself require additional adaptation (e.g., scaling divisor computation based on both geometry ratio and stash bucket count rather than stash bucket count alone) to achieve universal optimality across all allocation profiles?

- Can implementation efficiency gains (cycle 0009's in-place counter optimization for latency) and parameter co-optimization (cycle 0010's demotion gating) be combined without interference, or do they compete for the same performance budget?

## Per-policy notes

- All tested strategies (geometry allocation, promotion thresholds, demotion cascade gating, eviction candidate ranking) operate under LRU eviction policy on workload-a (Zipfian access pattern). No alternative eviction policies (LFU, ARC, CLOCK, etc.) have been explored.

- Frequency information is valuable for tier-routing decisions (where items move; cycle 0010's demotion gating) but not for eviction candidate selection (which items leave). This distinction separates successful and failed frequency-based approaches.

- Hit-rate preservation remains the hardest metric to maintain under aggressive optimization. Latency improvements are more readily decoupled from capacity decisions; hit-rate degradation indicates crossing into unfavorable capacity regime or excessive promotion churn.

- Promotion selectivity (threshold divisor) must scale inversely with geometry aggressiveness for stable composition. The relationship is mechanically coupled, not orthogonal; ignoring the coupling triggers cascading failures (cycles 0006–0007).
