# Cycle 0005 Insight

## Cohort outcome

This cycle attempted three distinct refinements to cycle 0003's occupancy-responsive thresholds: loosening ghost-queue retention to recover hit-rate (v001), reducing per-operation adjustment overhead by eliminating low-occupancy resizing (v002), and preserving frequency information across ghost entries without decay (v003). None achieved dual-metric improvement. v002 recovered latency gains but sacrificed hit-rate; v001 and v003 degraded both metrics, revealing that further refinement of threshold and ghost-queue mechanics runs against binding constraints.

## Patterns

- **Ghost-queue frequency preservation without decay semantics corrupts admission signals**: Both v001 (loosened retention threshold to freq ≥ -1) and v003 (full frequency preservation) attempted to improve hit-rate by retaining frequency information in ghost. Both failed because without proper decay discipline, the ghost queue accumulated inflated hotness candidates that corrupted main-queue admission decisions. v001's boosted re-admission weight (freq+2) proved insufficient to reflect actual utility; v003's preserved frequency conflated evicted items with genuinely hot ones. The coupling between retention policy, decay schedule, and re-admission weight is tight—tuning one lever alone destroys performance.

- **Structural per-operation overhead is the primary cost of occupancy-aware adaptation**: v002 demonstrates that v004's occupancy-aware sizing penalty came primarily from adjustment *frequency*, not magnitude. Removing the low-occupancy growth check (0.5 threshold) recovered latency (get –43ns, put –64ns) but forfeited hit-rate improvement. This reveals a fundamental trade-off: occupancy-responsive adaptation requires per-operation tracking, and that overhead cost cannot be eliminated without sacrificing the hit-rate benefit it enables.

- **Threshold adaptation (cycle 0003) remains superior to structural adaptation**: No variant improved upon cycle 0003/v003. All three moved in different directions—loosening ghost retention, reducing adjustment frequency, preserving frequency—but all failed, suggesting occupancy-responsive promotion thresholds represent a local optimum that incremental refinement cannot escape. Attempts to layer additional mechanisms on top either added overhead or degraded the admission signal.

## Open questions

- Can ghost-queue frequency preservation work with *stricter* decay semantics (decay to -2 or -3 before discard) than v001's approach, preserving more re-access opportunities while maintaining signal fidelity?

- Is the latency–hit-rate trade-off in structural sizing (v002) unavoidable, or could batch resizing, deferred adjustment, or alternative occupancy signals reduce per-operation cost while retaining hit-rate benefits?

- Does cycle 0003/v003 represent a true local optimum, or does the search space contain unexplored parameter combinations (e.g., different occupancy thresholds, promotion-frequency decay schemes) that could break the constraint frontier?

## Continuation

The cohort confirms that **occupancy-responsive thresholds are a local optimum beyond which incremental parameter or structural tuning cannot improve**. All three variants attempted refinements but all failed, validating cycle 0003/v003 as the strongest baseline. Ghost-frequency preservation requires decay discipline too restrictive to preserve hit-rate (v001) or incurs structural overhead (v003); occupancy-aware sizing couples latency overhead to hit-rate gain inseparably (v002). **For the next cycle:** (1) if hit-rate recovery remains critical, treat ghost-queue decay rate and retention threshold as fine-grained parameters (e.g., decay windows from -1 to -4) rather than structural redesigns; (2) if latency optimization is the priority, accept v002's reduced-adjustment approach as the most overhead-efficient within the current design, understanding hit-rate cost as the price of per-operation elimination; (3) decide whether to invest further in threshold-based admission or pivot to a fundamentally different mechanism (e.g., recency-weighted, bimodal thresholds, or decay-based ranking) to escape the current frontier; (4) before testing new mechanisms, exhaust fine-grained occupancy-threshold parameter spaces (e.g., thresholds at 0.4, 0.6, 0.8) to confirm whether cycle 0003/v003's parameters are optimal or whether tuning them can recover dual improvement without structural cost.
