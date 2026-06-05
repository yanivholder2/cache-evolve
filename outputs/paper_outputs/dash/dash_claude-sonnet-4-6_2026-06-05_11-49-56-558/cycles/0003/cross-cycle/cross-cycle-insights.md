# Cross-Cycle Insights

## Recurring wins

- **Hot-tier topology expansion is the single highest-leverage structural lever.** Rebalancing normal/stash from the baseline toward a hot-heavy split delivered the largest gains in cycles 1 and 2. The gain curve ascended through 50/50 and 75/25 before peaking at 24/8; stash can be reduced substantially without hurting cold-item re-entry until it drops to ~4 slots.
- **Position-based recency is the dominant eviction signal on workload-a.** Every variant that preserved or enhanced item-position ordering (classic LRU, MRU re-entry) beat or matched the baseline; no frequency-based variant has outperformed it in aggregate.
- **MRU re-entry (Back-insert after stash hit) is a reliable small positive.** It has produced additive gains in both cycles it was tested (cycles 2 and 3). It never hurt and never interfered with topology changes, even when its marginal contribution shrank as hot-tier capacity grew.
- **Confirmed optimal geometry: 24 normal / 8 stash buckets.** This is now a validated baseline for workload-a; future variants should build on it rather than re-probe topology.

## Recurring failures

- **Frequency-based eviction keeps underperforming on workload-a.** Standard LFU, aggressive aging-LFU, and threshold-gated aging-LFU have all been tested; none matched recency-ordered LRU in aggregate. The workload's access distribution makes position a stronger future-access predictor than any frequency counter variant tried so far.
- **Aggressive co-resident decay (−1 per hit to all peers) evicts mildly-warm items prematurely.** Tested in cycle 2; regressed. The penalty falls on useful middle-tier items before frequency counters can meaningfully differentiate them.
- **Softer threshold-gated aging does not rescue the core problem.** Tested in cycle 3; slow counter ramp-up creates an early-life eviction blind spot that replaces the aggressive-drain failure mode with a different one. The underlying issue is that frequency counters are a weak signal for this trace, not a calibration problem.
- **Over-expanding the hot tier past 24 slots regresses.** 28/4 caused a measurable drop; below ~8 stash slots, cold-item re-entry is harmed enough to outweigh the additional hot-tier pressure.

## Open questions

- **What unexplored structural axes remain?** Ghost/victim caches, explicit admission control, and scan-resistant insertion policies have not been probed. These are the natural next directions now that topology and re-entry position are near-exhausted.
- **Is there a more nuanced re-entry formulation?** Full Back-insert is nearly subsumed by the enlarged hot tier. A proportional insertion position (weighted by access count) might extract additional signal from the re-entry moment without being redundant.
- **Is frequency-based eviction structurally incompatible with workload-a, or just poorly calibrated?** If the access pattern is fundamentally recency-dominant, frequency discrimination should be retired for this trace. If it is a calibration issue, the threshold for "warm enough to apply decay" needs a much finer grain.
- **Does the confirmed 24/8 topology remain optimal under a different workload trace?** All wins so far are workload-a-specific; generalisability is untested.

## Per-policy notes

- **Classic LRU (baseline policy):** Robust and hard to beat on workload-a; recency ordering consistently dominates frequency variants. Serves as the correct policy anchor for this workload.
- **MRU re-entry / Back-insert:** Produces a small, consistent, non-interfering gain across cycles. Reliable as a compounding ingredient but not a standalone win. Contribution shrinks as hot-tier capacity grows.
- **Standard LFU:** Narrow win on workload-a in cycle 1 due to frequency skew, but not robust; its gain is workload-contingent and it should not be treated as a general improvement.
- **Aging-LFU (aggressive, −1 co-resident decay):** Regressed in cycle 2. Mildly-warm items are collateral damage; drop this variant.
- **Aging-LFU (threshold-gated / softer decay):** Regressed in cycle 3. Slow counter ramp-up is a structural flaw on workload-a, not a tuning issue. Retire frequency-based eviction for this trace.
- **Normal/stash bucket topology:** Gain curve ran 16/16 → 16/16 (50/50 rebalance, cycle 1 winner) → 24/8 (cycle 2 winner) → 28/4 (regression, cycle 3). Curve is confirmed exhausted; 24/8 is the optimum.
