# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0004

## Winners

- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0004/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0004/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0004/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0004/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_05-09-43-685/cycles/0004/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three winners pursued tighter admission filtering and queue resizing to reduce main-queue population and contention, achieving significant per-operation latency gains (28–66% faster operations across variants). However, all three winners trade hit-rate for these latency wins—v001 shows marginal miss impact, while v002 and v003 incur 8%+ hit-rate declines.

## Patterns

- **Latency-first trade-off**: Every variant in this cohort improves operation latency by constraining main-queue size, either through stricter promotion thresholds (v001, v002) or queue resizing (v003). The consistency suggests that reduced main-queue contention and lookup chains are a reliable latency lever.

- **Hit-rate as the cost**: The cohort reveals a fundamental tension: filtering that reduces main-queue noise also blocks items that would have been retained by looser admission. Ghost-hit re-routing (v002) and higher promotion thresholds (v001) both penalize moderately-accessed items that cannot meet stricter validation gates.

- **Workload sensitivity**: v003's unexpected latency improvement despite hit-rate decline suggests the workload benefits from smaller resident set and less eviction overhead. The three variants succeed at different severity levels, implying the trade-off frontier is workload-dependent.

## Open questions

- Can we recover hit-rate without sacrificing the latency gains—e.g., through smarter promotion heuristics or frequency-count decay?
- Do these variants cluster differently on synthetic vs. real-world access patterns?
- Is there a sweet spot where modest latency sacrifice recovers meaningful hit-rate, or is the frontier binary?

## Continuation

The cohort converges on **admission-filter tightness as a primary latency lever**. If latency is the priority metric, these variants form a solid foundation—v001 with marginal hit-rate cost is the most balanced, while v002 and v003 are extremes. **For the next cycle, either:** (1) accept the latency-first stance and optimize filter parameters further (frequency thresholds, queue sizing ratios); (2) explore frequency-decay or recency-weighted heuristics to recover hit-rate within the same tighter-filtering framework; or (3) test adaptive thresholds that adjust per workload phase to balance both metrics. Avoid looser admission filters; the cohort strongly signals they backfire.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
