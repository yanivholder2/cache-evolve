# Cycle 0003 Insight

## Cohort outcome

This cycle tested three distinct mechanisms to escape the latency-hit-rate tradeoff observed in previous cycles: separate ghost-queue tracking for frequency preservation (v001), strict threshold combined with ghost re-routing (v002), and adaptive thresholds scaled by queue occupancy (v003). Only v003 succeeded: it achieved substantial latency gains without sacrificing hit-rate, demonstrating that promotion decisions can be made responsive to queue state rather than fixed or structural.

## Patterns

- **Structural ghost-queue separation doesn't translate to hit-rate recovery**: v001's attempt to track warm-ghost items with preserved frequency failed because items overflow from the warm queue faster than they get re-accessed, increasing overhead while delivering no hit-rate benefit. The isolation of frequency information into a side structure proved less effective than embedding it in the critical path of eviction.

- **Strict thresholds lock in latency but close off hit-rate**: v002 achieved latency gains by enforcing freq > 2 promotion, but when combined with additional barriers (ghost re-routing + small-queue reset at freq=0), the system became over-restrictive. Items required three or more accesses to graduate to main, destroying hit-rate. The problem was not the threshold itself but the compounding effect of multiple restrictive mechanisms.

- **Adaptive thresholds decouple latency and hit-rate**: v003's key discovery is that promotion thresholds should respond to queue occupancy. Loosening the gate (freq > 1) when main queue has spare capacity allows more items to graduate and improves hit-rate during normal operation. Tightening (freq > 3) under high occupancy protects latency during contention peaks. This responsiveness achieved latency improvements (get p50 248→140ns, put p50 1088→577ns) without eroding hit-rate.

## Open questions

- What aspect of v003's adaptive mechanism—the occupancy-driven threshold loosening, the specific occupancy breakpoints (50%, 80%), or the range of thresholds (1–3)—is most responsible for the hit-rate preservation? Can this be simplified or made more robust?

- Does v003's threshold adaptation create latency spikes or oscillation when occupancy fluctuates near the 50% or 80% boundaries, or is the transition smooth?

- Can occupancy-awareness be combined with frequency-preservation mechanisms (e.g., a corrected version of v001's warm-ghost tracking with less aggressive overflow) to push latency further while maintaining the hit-rate stability v003 achieved?

## Continuation

The cohort delivers a critical insight: **promotion thresholds can be made responsive to queue state, allowing latency gains without the hit-rate penalty that fixed tight thresholds incur**. This pivots the next cycle from "accept the tradeoff" to "optimize queue-state responsiveness." For the next cycle: (1) stress-test v003's occupancy thresholds across workload phases to ensure no latency oscillation or worst-case spikes; (2) explore whether frequency distribution (e.g., tail of hotness) or access-pattern burstiness could inform threshold decisions more granularly than occupancy alone; (3) experiment with combining v003's adaptive threshold with corrected ghost tracking (fixing v001's overflow logic) to see if frequency preservation can add hit-rate benefit without overhead. Hold v003's design as a baseline—the search space has shifted from static tuning to state-responsive mechanics.
