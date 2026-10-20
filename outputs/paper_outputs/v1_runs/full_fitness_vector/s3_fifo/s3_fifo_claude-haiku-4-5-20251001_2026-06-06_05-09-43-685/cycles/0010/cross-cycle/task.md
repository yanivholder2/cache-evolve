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


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle tested three hypotheses aimed at escaping the occupancy-responsive-threshold plateau confirmed in cycle 0005: finer-grained occupancy breakpoints (v001), multi-level ghost-queue frequency decay with pre-computed re-admission signals (v002), and age-aware promotion thresholds (v003). All three variants succeeded in narrowly defined ways but revealed complementary trade-offs: v001 demonstrated that further occupancy-parameter tuning yields negligible hit-rate recovery at per-operation cost; v002 uncovered that pre-computed ghost frequencies enable dramatic latency gains (49% faster puts) but require a tuned re-admission formula to preserve hit-rate; v003 showed that age-based promotion pressure, while intuitive, adds overhead without unlocking the expected frequency-starvation fixes. The cohort pivots understanding away from "refine occupancy thresholds" toward a new frontier: **pre-computed ghost-queue state can deliver latency improvements if coupled with an adaptive re-admission signal**.

## Patterns

- **Per-operation overhead is the primary cost of threshold refinement**: Both v001 (occupancy-ratio computation per eviction) and v003 (age-ratio computation and conditional branching) added latency despite targeting hit-rate recovery. v001's 6ns get latency penalty and v003's 35–244ns put penalties exceed their marginal gains. This confirms cycle 0005's finding that occupancy-aware *mechanisms* are more expensive than occupancy-aware *thresholds*. Threshold selection itself (cycle 0003) was overhead-free; any downstream logic compounds the cost.

- **Ghost-queue frequency pre-computation enables latency breakthrough, but the re-admission formula is brittle**: v002's architecture—storing (K, frequency) tuples in ghost and using direct arithmetic on re-admission—delivers compelling latency wins (get 248→175ns, put 1088→550ns). However, the fixed boost formula `ghost_freq + 3` overshoots for some decay trajectories and undershoots for others, causing hit-rate loss (0.7164→0.7074). This reveals that the bottleneck is not mechanism or architecture but signal accuracy: the decay-to-actual-hotness mapping is workload-dependent.

- **Fine-grained occupancy-parameter tuning (0.4, 0.7 vs. 0.5, 0.8) does not escape the cycle 0005 local optimum**: v001's refined breakpoints neither recovered hit-rate nor reduced latency compared to cycle 0005/v002. This suggests occupancy thresholds are less sensitive to granularity than to the underlying policy. The cycle 0003/v003 design (threshold range 1–3, breakpoints 0.5/0.8) was well-calibrated for this workload; further tweaking explores noise, not new optima.

- **Age-aware thresholds (position in small queue) do not match the frequency-starvation pattern in this workload**: v003's hypothesis—that items aging out of the small queue deserve a looser promotion gate—intuited a real problem (items with freq=1 evict to ghost before reaching freq=2) but the proposed solution (loosening to freq > 1 for the back 30%) did not unlock additional main-queue promotions. Either the starvation scenario is rare in this access pattern, the back-30% window misses the vulnerable population, or frequency acquisition outpaces aging in the small queue, rendering the rescue mechanism unused.

## Open questions

- Can v002's fixed re-admission boost be replaced with an adaptive formula (e.g., `boost = 2 + occupancy_ratio` or `boost = 2 + frequency_percentile(ghost_freq)`) that preserves latency gains while tuning hit-rate to match or exceed baseline?

- Is v002's two-level decay (-2 threshold before discard) optimal, or should the depth be varied (e.g., -1 for aggressive filtering, -3 for lenient retention) to find a sweeter point between latency and hit-rate?

- Do v001 and v003's per-operation overhead costs have structural alternatives—e.g., sampling-based occupancy tracking, lazy evaluation, or amortized computation—that could reduce overhead without sacrificing the decision quality they target?

- Has the cohort reached a true Pareto boundary where further latency improvement necessarily degrades hit-rate, or does an unexplored mechanism (e.g., recency weighting in ghost, hybrid frequency-recency scoring) exist that could improve both?

## Continuation

The cohort identifies a promising new direction: **pre-computed ghost-queue state (v002's architecture) can unlock latency improvements, but the re-admission signal must adapt to workload characteristics**. Cycle 0003's occupancy-responsive thresholds remain the strong baseline for balanced performance, but v002 reveals a new frontier in ghost-queue efficiency. The per-operation overhead of occupancy-aware mechanics (v001, v003) is likely unavoidable; further gains from threshold tuning alone are unlikely. **For the next cycle:** (1) prioritize tuning v002's re-admission boost formula—test occupancy-responsive or frequency-distribution-aware boost scaling to recover hit-rate while retaining latency wins; (2) experiment with v002's decay depth (e.g., -1, -2, -3) as a coarse parameter sweep to isolate whether hit-rate loss is an artifact of too-aggressive pruning; (3) if dual-metric improvement remains critical and tuning v002's formula fails, consider whether a fundamentally different re-admission signal (recency, frequency percentile, decay-rate adaptation) could supplement or replace the current arithmetic boost; (4) if accepting latency-hit-rate tradeoffs is acceptable (latency-optimized deployments), v002's current architecture is near-optimal and should be tuned for that use case. Hold cycle 0003/v003 as the dual-metric baseline; use it as the decision point for whether to commit to v002's latency specialization.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle tested three distinct approaches to advance beyond cycle 0006's latency breakthrough: adding ghost-frequency decay to a lean baseline (v001), replacing the fixed ghost re-admission boost with occupancy-responsive scaling (v002), and using recency instead of frequency as the ghost re-admission signal (v003). Only v002 delivered clear dual-metric wins—v001 reintroduced per-operation overhead without recovering hit-rate, while v003 achieved dramatic latency gains at the cost of hit-rate loss. The cohort confirms that adaptive re-admission signals outperform both fixed formulas and frequency-decay approaches when paired with the right architectural foundation.

## Patterns

- **Occupancy-responsive adaptive boosting outperforms fixed-boost policies**: v002 proves that scaling ghost re-admission strength with main-queue occupancy delivers both latency improvement and hit-rate stability, extending cycle 0003's success with occupancy-aware thresholds into the realm of signal adaptation. The variable boost eliminates thrashing that fixed +3 formulas trigger during capacity transitions.

- **Recency is computationally efficient but predictively weak**: v003 demonstrates that entry position in ghost requires cheaper arithmetic than frequency lookup, yielding 43–57% latency improvements. However, recency alone is a poor proxy for re-access likelihood in this workload—temporal distance from eviction does not capture the same patterns that frequency or occupancy-based signals do, resulting in hit-rate loss.

- **Ghost-frequency decay cannot substitute for structural adaptation on lean baselines**: v001's attempt to add frequency preservation (-3 depth) to cycle 0005/v002's lean approach failed on both fronts: the per-operation overhead of ghost entry search and boosting contradicted the lean design, while the passive recovery mechanism could not replace the active barrier adaptation that structural sizing provides.

- **Pre-computed ghost-queue state with stored metadata remains the durable architectural choice**: All three variants leveraged ghost entries containing frequency or state information, confirming cycle 0006/v002's architectural insight that pre-computation enables efficiency. The critical variable is the re-admission signal used to exploit that state.

## Open questions

- Does combining v002's occupancy-responsive formula with recency-based filtering (e.g., prioritizing re-admission of recently-evicted items) recover both v002's latency and v003's hit-rate potential?

- Can v003's recency-to-boost scaling coefficients be tuned (e.g., 2–5 range instead of 2–4) to recover the hit-rate loss while retaining latency gains, or is the predictive gap structural?

- Is there a sweet spot for ghost-frequency decay depth (between -1 and -3) that reduces per-operation overhead while preserving enough warm candidates to improve hit-rate on lean baselines?

## Continuation

The cohort validates that **adaptive re-admission signals scale with workload state better than fixed formulas**, making v002 the strongest baseline for balanced performance. Recency (v003) opens a new latency frontier but at measurable hit-rate cost; the next cycle should explore whether signal combination or tuning recovers both gains. Ghost-frequency decay on lean architectures (v001) consistently reintroduces overhead costs that offset any hit-rate benefit—this direction should be deprioritized unless paired with structural changes that justify the per-operation cost. **For the next cycle:** (1) integrate v002's occupancy-responsive boost as the new baseline if latency-hit-rate balance is the goal; (2) if latency specialization is acceptable, v003's recency approach is near-optimal and worth tuning for that use case; (3) explore hybrid re-admission signals (frequency + recency, occupancy + recency) to test whether combining signals recovers v003's latency while protecting hit-rate; (4) avoid further ghost-frequency-decay variants unless they couple to structural mechanisms (e.g., dynamic sizing, adaptive thresholds) that actively use the preserved state.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle tested three distinct ghost-queue refinements aimed at escaping the latency-hit-rate plateau: selective frequency filtering to reduce ghost noise (v001), dual-signal re-admission combining frequency and recency (v002), and occupancy-gated retention to adapt ghost capacity under load (v003). All three variants succeeded in narrowly defined ways, confirming that improvements exist but at mutual trade-offs. v001 achieved dramatic latency gains for puts (63% faster) by filtering ghost to high-frequency items only, v002 explored signal combination for prediction accuracy but suffered from suboptimal weighting, and v003 revealed that occupancy-responsive mechanics are expensive on hot paths despite intuitive appeal.

## Patterns

- **Ghost-queue signal purity vs. diversity is a false dichotomy**: v001 demonstrates that filtering ghost to only freq > 0 items improves put latency by concentrating on genuinely warm candidates, but the hit-rate loss (0.61%) reveals the diversity cost—items that never reached admission thresholds but might re-access are discarded. Pure signal fidelity collapses hit-rate; pure diversity (retaining freq=0) includes noise that degrades prediction. The sweet spot lies between these extremes.

- **Multi-signal re-admission is mechanically sound but formula-brittle**: v002's architecture of combining frequency depth and recency position via normalized signals is conceptually strong—two orthogonal dimensions should better predict re-access than either alone. However, equal weighting (0.5/0.5) and conservative boost scaling (range 2–4) undershoots actual re-admission needs, suggesting that signal weights are workload-dependent and require fine-tuning or adaptive scaling.

- **Occupancy-responsive mechanics compound hot-path overhead**: v003's attempt to gate ghost retention with occupancy ratios introduces multiple division and branching operations into eviction paths, especially in the small queue where frequency is highest. The result: get p99 improves locally (smaller ghost pool to search), but put p99 spikes significantly due to branching overhead when small queue fills frequently. Occupancy sensitivity is valuable, but computing it on every eviction is too expensive.

- **Eviction-time computation is zero-cost; hot-path computation is not**: v001's ghost filtering happens at eviction time (infrequent) with zero hot-path impact; v003's occupancy-ratio computation happens on every eviction. This distinction is critical: architectural adaptations that defer complexity to cold paths (eviction) scale; those that add branching to hot paths (especially small-queue puts) do not.

## Open questions

- Can v001's ghost-admission threshold be relaxed (e.g., freq >= -1 instead of freq > 0) to recover lost hit-rate candidates while retaining most of the latency benefit?

- Does v002's dual-signal weighting respond to occupancy state—should recency weight increase under low occupancy (more re-access time) and frequency weight increase under high occupancy (less time to prove utility)?

- Can v003's occupancy computation be amortized (e.g., computed once per N evictions) or moved entirely to eviction-time decisions (like v001's filtering) to avoid hot-path branching?

- Has the cohort reached a Pareto frontier where latency improvements require hit-rate trade-offs, or does an unexplored mechanism (e.g., frequency-decay scheduling independent of eviction, sampling-based occupancy signals) exist that could improve both?

## Continuation

The cohort validates that **ghost-queue refinements unlock improvements, but each comes with a cost trade-off tied to computational placement and parameter tuning**. V001 proves filtering at eviction time is overhead-efficient; v002 shows signal combination is promising but needs formula refinement; v003 demonstrates occupancy sensitivity is expensive on hot paths. None achieved dual-metric wins, confirming that meaningful gains require accepting latency-hit-rate trade-offs or finding unexplored mechanisms. **For the next cycle:** (1) prioritize v001's threshold tuning—test freq >= -1 or freq >= 0 to find the boundary where diversity recovery outweighs noise cost; (2) if pursuing v002's dual-signal approach, explore adaptive weighting (e.g., occupancy-responsive boost scaling) or frequency-distribution-aware normalization to improve the prediction formula; (3) abandon occupancy-gated mechanics on hot paths (v003's approach); if occupancy sensitivity is needed, move it entirely to eviction-time filtering or deferred updates (batch occupancy computation); (4) decide whether latency or hit-rate is the primary metric for the next cycle—the current frontier appears to require trade-offs, making explicit prioritization essential for coherent exploration.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cycle tested three approaches to advance the ghost-queue frontier: relaxing the ghost-queue filter to recover lost hit-rate diversity (v001), moving occupancy-responsive boost computation to the ghost-hit cold path to eliminate hot-path overhead (v002), and batch-amortizing frequency decay to enable richer ghost state without per-operation cost (v003). All three variants succeeded in narrowly defined ways: v001 confirmed that aggressive filtering was correct by showing relaxation reintroduced overhead without gain; v002 proved that occupancy-responsive signals can be computed on cold paths with no per-operation penalty but revealed the boost formula requires tuning; v003 achieved dramatic latency wins (44% faster gets, 37% faster puts) while maintaining hit-rate by pushing decay entirely into amortized cold-path work. The cohort pivots understanding toward a new frontier: **computational placement (hot vs. cold path) is more valuable than mechanism innovation, and amortization unlocks richer state without cost.**

## Patterns

- **Ghost-queue signal purity was correctly calibrated**: v001's attempt to relax the freq > 0 filter to freq >= -1 catastrophically backfired, degrading put latency by 378ns p50 and 3405ns p99 without recovering any hit-rate. This confirms that cycle 0008/v001's aggressive filtering was not overly restrictive—freq=0 items are genuine noise that clutters the ghost lookup space, not hidden re-access opportunities. The boundary test reveals that further filtering, not relaxation, remains the path forward.

- **Occupancy-responsive adaptation is architecturally sound but formula-brittle**: v002 delivers a 19% get latency improvement by moving occupancy computation from the hot put path (where it incurs overhead) to the cold ghost-hit re-admission path (where it is negligible). This validates cycle 0008's insight that "eviction-time computation is zero-cost; hot-path computation is not." However, the quadratic boost formula `(2.0 + occupancy_ratio²)` is miscalibrated, providing excessive re-admission protection (freq 2–3) that crowds out small-queue working-set items, causing a 1.55% hit-rate regression. The architecture holds; the signal magnitude requires workload-specific tuning.

- **Amortization enables richer ghost state without per-operation cost**: v003 proves that expensive operations—like per-operation frequency decay—can be preserved if batched sufficiently (every 32 evictions). This delivers dramatic latency wins (44% faster gets, 37% faster puts) while maintaining hit-rate integrity (−0.05%, negligible noise). The result extends cycle 0008/v001's "zero-hot-path-overhead" philosophy: you can have both latency and richer ghost metadata if the decay window aligns with workload patterns. Frequency depth is preserved, not discarded; it is simply updated less frequently.

- **Hot-path vs. cold-path computation is the primary leverage point**: Both v002 and v003 succeed by deferring expensive operations (occupancy computation, frequency decay) to cold paths. Conversely, v001 fails because search-path overhead cannot be eliminated. This reframes the frontier: cycles 0006–0008 explored *which* signals to use; cycle 0009 shows the next lever is *where to compute them*. Mechanism innovation has reached diminishing returns; architectural placement is now the primary frontier.

## Open questions

- Can v002's quadratic boost formula be recalibrated (e.g., lower magnitude, occupancy-threshold gating, or frequency-distribution-aware scaling) to recover hit-rate while preserving the 19% latency gain?

- Does combining v002's occupancy-responsive boost with v003's batch-amortized decay unlock dual-metric gains, or do the two mechanisms interact negatively?

- Is v003's 32-eviction amortization window optimal for this workload, or should it be tuned (e.g., 16, 64) to balance decay freshness against amortization overhead?

- Can v002's ghost-hit re-admission selectivity be refined (e.g., only re-admit items with `ghost_freq > threshold(occupancy)`) to avoid crowding the small-queue working set while retaining occupancy-responsive adaptation?

## Continuation

The cohort validates that **computational placement (hot vs. cold path) is more valuable than mechanism innovation**, with amortization as the enabling technique for richer state. v003's batch-amortized decay is the strongest baseline for latency, while v002's architecture (occupancy-responsive cold-path computation) remains valid if the boost formula is tuned. v001's failure confirms that ghost-queue signal purity was correctly calibrated; further relaxation explores noise, not gain. **For the next cycle:** (1) prioritize tuning v002's boost formula—test lower magnitudes (e.g., 1.5 + occupancy_ratio) or frequency-threshold gating to recover hit-rate while retaining latency; (2) explore combining v003's batch decay with v002's occupancy-responsive re-admission to test whether richer state + adaptive signals unlock dual-metric wins; (3) if latency specialization is the goal, v003's amortized-decay approach is near-optimal and ready for production deployment; (4) validate v003's 32-eviction amortization window via parameter sweep (16, 32, 64) to ensure it matches this workload's small-queue eviction frequency; (5) hold v002's cold-path-computation insight as the architectural baseline—it has proven overhead-free and should be the foundation for signal adaptation going forward.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

This cycle tested three refinements aimed at escaping the latency-hit-rate plateau: linear occupancy scaling to weaken re-admission protection and recover hit-rate (v001), occupancy-responsive boost combined with batch-amortized decay (v002), and frequency-threshold gating on ghost-hit re-admission to prevent main-queue crowding (v003). None achieved dual-metric wins. v001 lost both latency and hit-rate by underprotecting re-admitted items. v002 failed to preserve the cold-path overhead advantage despite careful placement. v003 succeeded in improving hit-rate (+1.14%) but at severe latency cost (36% get slowdown). The cohort reveals that **mechanism soundness does not guarantee implementation efficiency**: all three tested architecturally justified approaches that foundered on computational cost.

## Patterns

- **Occupancy-responsive boost magnitude is tightly calibrated to workload; linear scaling misses the mark**: v001's switch from quadratic (2.0 + r²) to linear (1.5 + 2.0r) scaling reduced the boost range from 2–5 to 1.5–3.5. This weakened re-admission protection, losing both the latency advantage and failing to recover hit-rate. The failure suggests the issue is not the quadratic formula's magnitude but its calibration relative to queue dynamics. Linear scaling underprotects re-access patterns in this workload, confirming that the space between linear and quadratic remains unexplored and may require non-standard functional forms.

- **Cold-path architectural advantage collapses under computational complexity**: v002 combined cycle 0009/v002's occupancy-responsive boost with cycle 0009/v003's batch-amortized decay. Despite placing occupancy computation at ghost-hit re-admission (a cold path), the floating-point arithmetic—division, squaring, min operation—introduced measurable overhead (31% get slowdown, 12% put slowdown). The "zero-cost cold-path" principle from cycle 0009 breaks when the computation itself is inherently expensive. Cold paths are only zero-cost if operations are cheap.

- **Frequency-threshold re-admission selectivity is architecturally sound but implementation-expensive**: v003's selective re-admission (only re-admitting items with demonstrated re-access patterns) delivered a clear hit-rate win (+1.14%) by preventing low-frequency ghost entries from crowding the small-queue working set. However, the implementation—extended ghost tuple structure `(K, u8)` and frequency lookup on every put—increased get latency by 36% and put by 6.8%. The mechanism is correct; the overhead is unacceptable.

- **Parameter tuning has exhausted its yield; implementation efficiency is now the bottleneck**: Cycles 0007–0009 progressed by exploring *which* signals to use and *where* to compute them. Cycle 0010 shows that refinements to those mechanisms yield only trade-offs. v001's parameter adjustment failed; v002's architectural reuse failed; v003's mechanism refinement improved one metric at severe cost to another. The frontier has shifted from signal innovation to implementation optimization.

## Open questions

- Can v003's frequency-threshold gating be optimized—e.g., by storing frequency inline in ghost entries, using bit-packed flags, or applying the check only to a subset of ghost-hit cases—to recover latency while retaining the +1.14% hit-rate improvement?

- Does the observed trade-off pattern indicate that the ghost-queue evolution has reached a Pareto frontier, or do alternative architectural approaches (e.g., adjusted small/main queue sizing ratio, different eviction order) exist that could improve both metrics simultaneously?

- For cycle 0009/v003's fixed +2 boost baseline (44% get latency, 37% put latency wins), what is the opportunity cost of the lost 1.14% hit-rate? Is the latency specialization acceptable for the workload, or must hit-rate recovery be prioritized?

- Can frequency-selective re-admission be applied selectively (e.g., only when main-queue occupancy exceeds a threshold) to reduce the hot-path overhead while preserving the crowding-prevention benefit?

## Continuation

The cohort validates that **mechanism soundness does not guarantee implementation efficiency**, and the frontier has shifted from signal design to computational optimization. V003's +1.14% hit-rate gain proves frequency-selective re-admission is architecturally correct but too expensive as implemented. V002's failure shows that floating-point occupancy calculations carry hidden costs even at cold-path placement. V001 confirms that linear scaling underprotects this workload. **For the next cycle:** (1) **if hit-rate is the priority**, focus on optimizing v003's selective re-admission mechanism—test inline frequency storage or sampled checks (every Nth ghost hit) to measure whether latency can be recovered; (2) **shelve occupancy-responsive adaptation** for now—both cycle 0009/v002 and this cycle's v002 demonstrate that adaptive boost formulas carry implementation overhead that erases theoretical cold-path advantage; (3) **validate the Pareto frontier** by testing structural alternatives (queue sizing, eviction order) rather than parameter tuning, since five cycles of refinement have yielded only trade-offs; (4) **establish a metric priority decision**: cycle 0009/v003's baseline (44% latency wins, negligible hit-rate loss) may be the production-ready solution if latency specialization is acceptable; otherwise, commit resources to optimizing v003's selective re-admission to unlock hit-rate gains without overhead cost.


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
