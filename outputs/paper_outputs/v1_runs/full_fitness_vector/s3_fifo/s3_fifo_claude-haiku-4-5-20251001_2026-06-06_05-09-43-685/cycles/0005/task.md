# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0005

## Winners

- v002
- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants pushed v001's tight admission-filter strategy in different directions—raising the promotion threshold, expanding the small queue, and deferring main-queue eviction—seeking to recover hit-rate or further improve latency. All three succeeded as winners by discovering distinct latency-performance corners: v001 optimized median put efficiency at the cost of tail-latency unpredictability, v002 (surprisingly) added small-queue capacity despite reduced hit-rate and search overhead, and v003 traded get latency for put efficiency through lazy eviction. None recovered hit-rate meaningfully, confirming that the admission filter's strict threshold—not queue sizing or eviction timing—is the binding constraint.

## Patterns

- **Admission filtering remains non-negotiable**: All three hypotheses built on v001's freq > 2 (or freq > 3) threshold, and all three discovered that tweaking surrounding mechanisms (threshold level, queue ratio, eviction delay) don't fundamentally alter the hit-rate cost. The filter is the bottleneck, and tuning it in isolation (v001: freq > 3) or working around it (v002, v003) yields only indirect gains.

- **Operation-specific trade-offs dominate**: v001 improved put p50 but hurt get and put p99, revealing that median and tail latencies respond differently to the same filter. v003 flipped the trade: speeding puts while degrading gets. The cohort shows no variant improved both simultaneously—each optimizes one dimension at another's expense.

- **Simple queue-sizing doesn't recover hit-rate**: v002's expansion from 25% to 30% actually reduced hit-rate by increasing eviction churn before items could accumulate sufficient frequency. The interaction between queue capacity and promotion threshold is tighter than adjusting capacity alone can address; larger windows don't help if the threshold gate is too strict.

- **Lazy eviction addresses the wrong bottleneck**: v003's deferred main-queue eviction improved put latency by reducing eviction calls, but didn't improve hit-rate as hypothesized. Items still evicted immediately upon small-queue timeout, revealing that main-queue eviction frequency is not the constraint—admission filtering is.

## Open questions

- Can a smarter admission mechanism (frequency decay, bimodal thresholds, recency weighting) decouple hit-rate recovery from the strict promotion threshold?
- Is there a compound variation (threshold + queue ratio + eviction timing together) that balances get and put latency without sacrificing the tail?
- Do the winners' gains hold on workloads with broader frequency distributions, or is this tight filtering specific to Zipfian access?

## Continuation

The cohort unanimously points to **admission-filter tightness as a non-negotiable latency lever that carries an inherent hit-rate cost**. Further tuning the threshold level (v001), queue proportions (v002), or eviction timing (v003) moves latency around but doesn't escape the fundamental trade-off. **For the next cycle:** (1) explore smarter admission mechanisms beyond binary frequency thresholds—decay-based ranking, multi-level frequency buckets, or adaptive thresholds that respond to workload burstiness; (2) if pursuing operation-level latency balance, test compound tuning (e.g., freq > 2 with 30% queue + 85% eviction threshold) rather than isolated knobs; (3) consider whether the hit-rate penalty is acceptable as a cost of latency optimization, or prioritize frequency-preservation heuristics (ghost tracking, deferred promotion) to protect warm-but-not-hot items. Avoid further simple threshold or sizing tweaks in isolation—the search space is narrow.


### Cycle 0003

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


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle tested three mechanisms to refine v003's occupancy-responsive success from cycle 0003: frequency decay in the ghost queue (v001), frequency-distribution percentile thresholds (v002), and dynamic small-queue sizing (v003). The two winners diverged sharply: v001 achieved significant put-latency gains by aggressively filtering the ghost queue, but sacrificed hit-rate; v003 recovered hit-rate through dynamic occupancy-aware queue sizing, but incurred latency overhead. v002's attempt to use frequency distribution as a threshold signal failed, proving less robust than occupancy-based signaling. The cohort reveals a critical constraint: occupancy-awareness successfully decouples metrics at the threshold level (cycle 0003), but applying it to structural queue sizing introduces per-operation overhead that forces new trade-offs.

## Patterns

- **Occupancy-awareness scales poorly to structural changes**: v003's dynamic small-queue sizing successfully improved hit-rate (0.7162→0.7248) by adapting capacity during load phases, but the occupancy tracking and resizing logic on every put operation degraded both get and put latencies (get p50 +36ns, put p99 +86ns). Threshold adaptation (cycle 0003) was overhead-free; queue structure adaptation is not.

- **Ghost-queue filtering requires calibration, not just filtering logic**: v001's frequency decay successfully reduced ghost-queue overhead and accelerated puts (p50: 577→203ns), but the filtering was too aggressive—items were ejected before accumulating sufficient re-access probability. The mechanism shows promise (frequency preservation embedded in the critical path, not structurally separated as in cycle 0003's v001), but the decay rate or retention threshold needs loosening to preserve more ghost entries.

- **Frequency distribution is a less stable signal than occupancy**: v002's 75th percentile threshold proved too conservative for the workload, reducing hit-rate and adding sampling/percentile-computation overhead to critical paths. Unlike occupancy (which directly reflects queue state), frequency distribution requires workload-specific calibration and is sensitive to access-pattern shifts. Occupancy remains the more robust observable signal.

- **The cohort discovered an unresolved latency-hit-rate scaling problem**: Two winners each optimized one metric at the other's expense, departing from cycle 0003's v003, which balanced both. Neither variant recovered the dual-improvement seen in cycle 0003, suggesting the hypothesis space may have been too focused on single levers or the magnitude of structural changes too coarse.

## Open questions

- Can v001's aggressive decay filtering be loosened (higher retention threshold, longer decay window) to preserve more ghost-queue candidates without losing the latency gains from reduced ghost-queue size?

- Is v003's latency overhead in occupancy tracking structural and unavoidable, or does scaling down the sizing adjustment range (e.g., ±5% instead of ±15%) proportionally reduce overhead while retaining hit-rate benefits?

- Could a hybrid approach—v003's dynamic sizing logic paired with v001's gentler frequency-decay ghost-queue management—recover the hit-rate gains without the latency penalty, or would the combined overhead exceed v003's individual cost?

## Continuation

The cohort confirms that **occupancy-responsive thresholds (from cycle 0003) remain the most effective lever**, but scaling occupancy-awareness to queue structure incurs non-trivial overhead. v001's ghost-queue frequency-decay approach is mechanically sound but over-aggressive; the next cycle should focus on parameter tuning rather than new mechanisms. Avoid further frequency-distribution experiments unless paired with occupancy as a secondary signal. **For the next cycle:** (1) tune v001's decay parameters (retention threshold, decay window) to preserve more ghost entries while maintaining latency gains; (2) test v003 with reduced sizing-adjustment magnitudes (±5–10%) to isolate whether overhead is proportional; (3) if dual-metric improvement is critical, explore hybrid configurations (v003's sizing + v001's tuned decay) rather than new signal types; (4) if latency stability is the priority, accept that v001's put-latency gains are available at hit-rate cost, but require hit-rate recovery through ghost-queue tuning, not structural sizing.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
