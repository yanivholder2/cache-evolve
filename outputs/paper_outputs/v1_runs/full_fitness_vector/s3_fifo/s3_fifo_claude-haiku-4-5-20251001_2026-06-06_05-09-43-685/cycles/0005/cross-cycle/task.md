# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

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


### Cycle 0005

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


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
