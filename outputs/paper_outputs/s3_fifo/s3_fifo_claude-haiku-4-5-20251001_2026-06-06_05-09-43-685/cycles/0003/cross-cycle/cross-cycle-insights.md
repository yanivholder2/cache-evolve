# Cross-Cycle Insights

## Recurring wins

- **Admission-filter tightness consistently improves operation latency**: Across all three cycles, variants that constrain main-queue population via stricter promotion thresholds or frequency gates deliver 28–66% latency improvements. The effect is workload-robust, persisting as different mechanisms (threshold tweaking, queue resizing, adaptive gates) are applied.

- **Adaptive/state-responsive promotion thresholds decouple latency from hit-rate**: Cycle 3's v003 achieved the first latency gain without hit-rate penalty by tying promotion decisions to queue occupancy rather than fixed thresholds. This pivot from static tuning to state-aware mechanics represents the cohort's most impactful discovery—operations improve 40–50% while hit-rate holds stable.

- **Queue-state information yields better decisions than structural side-channels**: Embedding promotion logic in the critical path of eviction/occupancy (v003) outperforms isolated frequency-tracking structures (v001 cycle 3). Responsiveness to live queue state matters more than preserving frequency offline.

## Recurring failures

- **Fixed tight promotion thresholds inevitably trade hit-rate for latency**: Every variant enforcing strict static gates (freq > 2, freq > 3) across cycles 1–2 incurred measurable hit-rate decline (8%+ in many cases). Attempts to recover hit-rate through queue resizing, eviction deferral, or threshold tweaking in isolation all failed, confirming the threshold itself is the binding constraint.

- **Structural ghost-queue separation doesn't translate to hit-rate recovery**: Cycle 3's v001 attempted to preserve frequency information in a separate warm-ghost queue, but items overflow before re-access, increasing overhead with zero hit-rate benefit. Frequency preservation requires embedding in the eviction critical path, not isolation.

- **Simple queue-sizing and eviction-delay adjustments don't escape the fundamental trade-off**: Cycle 2's queue expansion (v002) and lazy eviction (v003) failed to recover hit-rate despite addressing orthogonal mechanisms. They move latency around but don't challenge the binding constraint—promotion filtering.

- **Compounding restrictive mechanisms accelerate hit-rate collapse**: Cycle 3's v002 stacked freq > 2 thresholds with ghost re-routing and frequency resets, creating over-restrictive barriers that destroyed hit-rate. Multiple filtering layers don't improve selectivity; they amplify penalty.

## Open questions

- **What aspects of v003's occupancy-responsive design are critical?** Which elements drive the hit-rate decoupling: the threshold loosening strategy (freq > 1 when occupancy < 50%), the specific breakpoints, the threshold range (1–3), or their combination? Can the mechanism be simplified without losing the benefit?

- **Does occupancy-driven promotion create latency oscillation or instability?** When queue occupancy fluctuates near decision boundaries (50%, 80%), do thresholds flip frequently enough to cause latency spikes, or is the transition smooth? How robust is v003 to phase changes in workload burstiness?

- **Can occupancy-aware thresholds be combined with corrected frequency-preservation?** If ghost-queue overflow (v001 cycle 3's failure mode) were fixed, could a hybrid of v003's adaptive thresholds with improved frequency tracking push latency further while maintaining hit-rate stability?

- **How do these patterns generalize beyond Zipfian access?** Are tight admission filtering and occupancy-responsive gates universally beneficial, or are they tuned to this workload's specific frequency distribution? Does burstiness or skew-level alter the frontier?

## Per-policy notes

- **Fixed tight-threshold policies (freq > 2, freq > 3)**: Deliver consistent 28–66% latency improvements but incur 8%+ hit-rate losses (cycles 1–2). Create operation-specific trade-offs (median vs. tail latency), where optimization in one dimension degrades another. The threshold acts as a non-negotiable binding constraint; downstream tuning (queue sizing, eviction timing) cannot recover hit-rate without loosening the gate itself.

- **Ghost-queue tracking and frequency isolation (cycle 3 v001)**: Fails to recover hit-rate despite explicit frequency preservation. Overflow dynamics (items leave warm queue faster than re-access windows allow) generate overhead without benefit. Demonstrates that frequency information must participate in the live eviction path, not reside in an isolated structure.

- **Adaptive/occupancy-responsive thresholds (cycle 3 v003)**: Achieves latency improvements (get p50 248→140ns, put p50 1088→577ns) without hit-rate penalty by dynamically adjusting promotion barriers based on queue occupancy. Loosens gates during underutilization to improve hit-rate, tightens during contention to protect latency. Marks a shift from fixed parameter tuning to state-responsive policy—the only variant to escape the latency–hit-rate frontier.

- **Queue-sizing and eviction-delay variations (cycle 2)**: Do not address the binding constraint of promotion thresholds. Expansion (v002: 25%→30%) increases eviction churn without improving hit-rate; deferral (v003 cycle 2) reduces eviction calls without recovering hit-rate. These knobs are orthogonal to the admission gate and should only be tuned in concert with promotion policy changes, not in isolation.
