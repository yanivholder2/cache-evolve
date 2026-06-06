# Cross-Cycle Insights

## Recurring wins

- **Occupancy-responsive thresholds achieve dual-metric improvement**: Cycle 0003 v003 demonstrated that promotion thresholds can be made responsive to queue state (loosening when capacity available, tightening under contention), delivering substantial latency gains (get p50 248→140ns, put p50 1088→577ns) without hit-rate erosion. This remains the only approach to date that improved both metrics simultaneously. Cycle 0004 confirmed that departing from occupancy-awareness to pursue other mechanisms (sizing, frequency distribution) lost this balance.

- **Tight admission filtering is a reliable latency lever across workloads**: All four cycles converged on stricter promotion thresholds or frequency gates as the primary driver of reduced main-queue contention and operation latency. The cost is consistent (hit-rate penalty) but the mechanism is robust.

- **Occupancy is a more stable signal than frequency distribution**: Cycle 0004 v002's attempt to use frequency-distribution percentiles proved less reliable than occupancy-based signaling. Occupancy directly reflects queue state; frequency distribution requires workload-specific calibration and is sensitive to access-pattern shifts.

## Recurring failures

- **Simple queue-sizing adjustments fail to address the binding constraint**: Across cycles 0002–0004, expanding queue capacity (v002 25%→30%, v003 dynamic sizing) either reduced hit-rate or added latency overhead without recovery. The issue is that admission filtering—not queue structure—is the bottleneck; resizing capacity does not change the threshold gate that blocks items.

- **Structurally separated ghost-queue tracking introduces overhead without benefit**: Cycle 0003 v001's attempt to isolate warm-ghost frequency information failed because items overflow faster than they re-access, increasing overhead while delivering zero hit-rate gain. Ghost mechanisms must be embedded in the critical path (as in cycle 0004 v001's frequency decay), not separated structurally.

- **Compounding restrictive mechanisms destroy hit-rate exponentially**: Cycle 0003 v002 combined strict freq > 2 threshold with ghost re-routing and small-queue reset at freq=0, effectively requiring freq > 3 for main-queue promotion. Each additional barrier reduced hit-rate multiplicatively. Avoid stacking restrictions.

- **Operations-critical-path structural changes incur non-trivial latency overhead**: Cycle 0004 v003's dynamic queue sizing improved hit-rate but added per-operation occupancy tracking and resizing overhead (get p50 +36ns, put p99 +86ns). Threshold adaptation (cycle 0003) is overhead-free; structural adaptation is not.

## Open questions

- **Ghost-queue frequency-decay calibration**: Cycle 0004 v001 showed decay-based filtering can reduce ghost overhead while accelerating puts, but the decay rate or retention threshold was too aggressive. What parameter range preserves sufficient ghost entries to recover hit-rate while maintaining latency gains?

- **Frequency preservation in the critical path**: How can frequency information be embedded effectively during eviction without the separation overhead of cycle 0003 v001 or the over-aggressiveness of cycle 0004 v001? Is there a middle ground with gentler decay or adjusted retention thresholds?

- **Structural overhead scaling**: Is the latency cost of occupancy-tracking in queue-sizing operations proportional to adjustment magnitude (e.g., ±5–10% vs. ±15%)? Can this be optimized to recover dual-metric gains without the cost of cycle 0004 v003?

- **Hybrid mechanism synergy**: Could combining cycle 0003 v003's dynamic sizing (tuned to lower overhead) with cycle 0004 v001's gentler frequency-decay ghost filtering recover hit-rate preservation without exceeding the individual component costs?

- **Alternative queue-state signals for threshold adaptation**: Beyond occupancy and frequency distribution, could access-pattern burstiness, tail-hotness ranking, or recency-weighted frequency inform threshold decisions more granularly?

## Per-policy notes

- **Occupancy-responsive thresholds** (Cycle 0003 v003, foundation for 0004): Loosen freq gate when main queue has spare capacity; tighten during contention. Only approach to achieve latency + hit-rate simultaneously. Remains the performance baseline; no variant has surpassed it.

- **Frequency-decay ghost filtering** (Cycle 0004 v001): Reduced ghost-queue overhead and accelerated puts (p50 577→203ns), but parameters were over-aggressive. Mechanism is mechanically sound; needs looser decay rate or higher retention threshold. Worth revisiting with parameter tuning rather than architectural change.

- **Frequency-distribution thresholds** (Cycle 0004 v002): Percentile-based signals proved unstable and sensitive to workload shifts. Less robust than occupancy. Avoid in future iterations unless paired with occupancy as secondary signal.

- **Dynamic queue sizing** (Cycle 0004 v003): Successfully recovered hit-rate (0.7162→0.7248) but traded latency overhead (get p50 +36ns, put p99 +86ns). The interaction between sizing adjustments and per-operation cost is tighter than anticipated; small adjustments (±5–10%) may be worth testing to reduce overhead.

- **Ghost re-routing with strict thresholds** (Cycle 0003 v002): Re-routing logic combined with multiple restrictive gates is over-constraining. Avoid compounding restrictions; if using ghost re-routing, keep main-queue promotion gate loose enough to avoid exponential hit-rate penalty.
