# Cross-Cycle Insights

## Recurring wins

- **Frequency-based eviction with periodic decay** is the consistently effective approach. Every winning variant across cycles 0001–0005 incorporates frequency tracking coupled with periodic multiplicative decay (0.75× at capacity × 16 intervals). This two-lever strategy—accumulating frequency over time and gradually forgetting historical patterns—successfully adapts to workload structure and phase shifts.

- **Decay curve shape matters more than decay frequency.** Cycles 0004–0005 revealed that the *form* of decay is critical: multiplicative decay (0.75×) consistently outperforms more aggressive approaches (0.5× right-shift from cycle 0004, 0.7× coefficient from cycle 0005). Smoother forgetting preserves frequency discrimination across workload transitions without losing historical pattern memory.

- **Simple, fixed parameters outperform adaptive complexity.** The fixed capacity × 16 decay interval and 0.75× coefficient beat every attempt at dynamic adjustment (cycle 0003 v002, cycle 0005 v003). Computational overhead and loss of interpretability from adaptive mechanisms do not justify marginal or nonexistent gains on this workload.

- **LRU tie-breaking within frequency tiers is correct.** Cycle 0002 showed that head-to-tail (oldest-first) victim selection within equal-frequency items is superior to reversal. Temporal locality persists within frequency classes; newer arrivals truly are less likely to be re-accessed than older ones.

## Recurring failures

- **Zone-based partitioning consistently backfires.** Every attempt to split the cache into cold/hot zones or apply frequency-dependent promotion thresholds (cycle 0001 v003 ghost-list, cycle 0002 v002 zone hybrid, cycle 0003 v001/v003 with independent decay rates) either failed or regressed. Artificial zone boundaries mask frequency signals and create promotion barriers that prevent moderate-frequency items from reaching hot-zone protection before eviction. Unified competition on frequency alone is superior.

- **Adaptive or threshold-dependent decay mechanisms add overhead without benefit.** Cycle 0003 v002 (frequency-distribution-aware decay thresholds) and cycle 0005 v003 (exponential approximation) both introduced complexity to adjust decay dynamically, yet failed to improve upon the parent strategy. The workload either lacks sharp phase transitions requiring real-time adaptation, or the cost of tracking statistics simply outweighs the benefit.

- **Accelerating decay frequency hurts performance.** Cycle 0004 v002 (doubling decay rate from capacity × 16 to capacity × 8) actively regressed by forcing the cache to forget historical patterns too aggressively. The original interval is robustly tuned for this workload's gradual phase shifts.

## Open questions

- **Does the 0.75× decay coefficient and capacity × 16 interval generalize across workloads and cache sizes, or are they tuned to this specific trace?** Both values have proven robust within this scenario, but portability is unclear. Sensitivity testing across different workload traces and cache capacities would reveal whether these parameters are universal or scenario-specific.

- **Is there exploitable margin via threshold-dependent or piecewise decay?** Cycle 0005 suggests the cache has converged to a tight local optimum around multiplicative decay. Could lower-cost alternatives—such as decaying only frequencies above a threshold, applying different rates to hot vs. cold items, or frequency-dependent decay coefficients—bypass the plateau without abandoning the baseline?

- **What aspect of the access pattern makes decay curve shape so critical?** Is the workload's frequency distribution multimodal (high sensitivity to forgetting rates), or are gradual phase shifts the dominant factor? Understanding the root cause could guide exploration in future cycles.

- **Could frequency quantization, tie-breaking refinements, or interval tuning yield further marginal gains?** Cycles 0004–0005 show diminishing returns from coefficient adjustment, but orthogonal levers (finer frequency granularity, more sophisticated victim selection within equal-frequency items, or capacity-dependent interval formulas) have not been fully explored.

## Per-policy notes

- **Frequency-based eviction evolved through five cycles into a stable, locally-optimal strategy.** The winning policy family is characterized by: (1) per-item frequency counters tracking access count; (2) periodic multiplicative decay at capacity × 16 intervals, applying 0.75× factor to all frequencies; (3) LRU-ordered victim selection within frequency tiers, scanning head-to-tail (oldest first). This design balances protection of hot items (via frequency), gradual workload adaptation (via periodic decay), and temporal locality (via LRU tie-breaking).

- **Simplicity and robustness are strongly preferred over adaptive mechanisms on this workload.** Cycle 0001 established that frequency protection works; cycle 0002 added periodic decay; cycles 0003–0005 confirmed that all subsequent attempts to introduce zone boundaries, dynamic thresholds, or aggressive adaptation either regressed or stagnated. The strategy has converged to a well-tuned fixed-parameter regime.

- **The cache is approaching a tight local optimum.** Cycle 0005's failure of all three perturbations (coefficient sensitivity in both directions and exponential approximation) signals the bottleneck for improvement is no longer within decay mechanics. Further gains likely require exploration outside the core eviction algorithm—e.g., access pattern prefetching, working-set detection, or multi-level cache hierarchies—or acceptance that this workload's hit-rate ceiling has been reached with frequency-based protection.
