# Cross-Cycle Insights

## Recurring wins

- **Frequency-based protection with periodic decay:** Every winning variant since Cycle 0002 has relied on tracking access frequency and periodically resetting counters. This core approach—protecting frequently-accessed items from eviction while gradually forgetting ancient hotspots—consistently outperforms alternatives.
- **Simple, fixed-parameter tuning beats adaptive complexity:** The `capacity × 16` decay interval (Cycle 0002) and the 0.75× multiplicative decay curve (Cycle 0004) both resist refinement; attempts to make them adaptive or context-sensitive (Cycles 0003 and 0004) degrade performance. The workload's structure appears to be captured by hand-tuned constants rather than real-time adaptation.
- **Unified frequency-based competition:** When all items compete on a single frequency signal without artificial zone boundaries, the cache reliably identifies truly hot items. Partitioning strategies that protect cold items separately (Cycles 0002, 0003) either add no value or actively hurt by masking frequency discrimination.

## Recurring failures

- **Zone-based partitioning and hybrid approaches:** Despite initial promise in Cycle 0001 v002 (~8.3% gain), every attempt to split the cache by access pattern (cold/hot zones, resized hot zones, independent decay rates) has either regressed or provided marginal benefit while adding complexity. Artificial promotion barriers prevent items from competing freely on frequency, trapping moderately-frequent items in the wrong zone.
- **Adaptive decay mechanisms:** Cycle 0003 v002 (adaptive frequency-distribution-aware thresholds) and Cycle 0004 v002 (accelerated decay intervals) both failed, suggesting the workload either lacks sharp phase transitions justifying real-time adaptation, or the tracking overhead outweighs phase-shift benefits. Fixed intervals and curves are more efficient.
- **Counter saturation as a lever:** Cycle 0004 v001 showed capping frequency counters at 32 has no measurable impact, indicating saturation is not a bottleneck in this workload—either frequencies don't reach saturation limits, or periodic decay refreshes priorities faster than saturation can accumulate.

## Open questions

- **Decay coefficient sensitivity:** Does the 0.75× multiplicative decay coefficient generalize across workloads and cache sizes, or is it tuned to this specific trace? How sensitive is performance to nearby coefficients (0.7×, 0.8×, 0.85×)?
- **Why zone partitioning consistently fails:** Is the workload fundamentally lacking structural properties (e.g., temporal phase shifts, clear hot/cold bimodality) that zone-based strategies could exploit? Or does the recency ordering (LRU within zones) introduce hidden overhead that frequency-unified approaches avoid?
- **Decay curve exploration:** Could patterns other than multiplicative decay (e.g., exponential, piecewise linear, or frequency-distribution-dependent decay) capture further gains beyond the marginal win observed from 0.75× over 0.5×?
- **Generalization of tuning:** How portable are the `capacity × 16` decay interval and 0.75× decay coefficient across different workloads, cache capacities, and access distributions?

## Per-policy notes

- **Frequency-based eviction:** Established as the dominant policy by Cycle 0002 and has remained the foundation of every winner since. Performance is highly sensitive to decay mechanics (interval and curve shape) but robust against counter saturation or tie-breaking refinements. The policy's strength lies in its simplicity: global frequency counters allow fair competition across the entire cache, avoiding the partitioning overhead that hurts simpler zone-based alternatives.
- **Periodic decay mechanism:** Introduced as a refinement in Cycle 0002 v003 and is now inseparable from the winning strategy. The mechanism captures temporal adaptation (shifting focus from ancient hotspots to current patterns) with minimal overhead. The `capacity × 16` interval is robustly tuned; accelerating it to `capacity × 8` actively degrades performance, and the 0.75× multiplicative curve outperforms harsher right-shift decay, suggesting gentler forgetting preserves discrimination between hot and cold items across phase transitions.
- **Zone-based and hybrid approaches:** Explored in Cycles 0001–0003 and abandoned after consistent underperformance. Even hybrid schemes (Cycle 0002 v002) that combined LRU for cold items with frequency-based selection for hot items underperformed the unified frequency approach. The underlying issue appears to be that zone boundaries create artificial barriers preventing items from demonstrating true frequency value.
- **Counter management:** Saturation capping is not a constraint; Cycle 0004 v001 showed no benefit from capping at 32 vs. natural saturation. Focus remains on decay interval and curve shape rather than saturation handling.
