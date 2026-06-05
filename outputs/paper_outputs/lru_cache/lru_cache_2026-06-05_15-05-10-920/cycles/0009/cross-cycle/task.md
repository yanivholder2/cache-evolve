# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

This cycle tested three strategies for improving hit rate by recognizing non-uniform access patterns: frequency-based eviction (v001, winner), zone-based partitioning (v002), and ghost-list "second chance" handling (v003). The winning frequency-based approach and the zone-partitioning strategy both significantly improved hit rate by protecting frequently-accessed items, while the ghost-list variant degraded performance through conservative re-insertion placement.

## Patterns

- Both high-performing variants (v001 and v002) converged on the same insight: the workload exhibits access-skewed patterns where some items are accessed far more frequently than others, and protecting these "hot" items from eviction is more important than pure recency.
- Frequency-based eviction (global counters) and zone-based partitioning (local promotion thresholds) achieved comparable gains (~9% and 8.3% respectively), suggesting multiple valid approaches to the same problem.
- The ghost-list strategy failed not because the core intuition was wrong, but because placing re-inserted items at the middle of the LRU order was too conservative—they should have been prioritized higher to match the observed benefit of protecting valuable items.

## Open questions

- Why does the frequency-based approach narrowly outperform zone-based partitioning on this workload? Is there a secondary effect (e.g., better cache locality, lower tie-breaking variance) that favors global counters?
- Could the ghost-list mechanism be salvaged by adjusting the insertion position closer to the tail, or by coupling it with frequency tracking?
- How do these strategies scale in terms of eviction cost and memory overhead as capacity grows?

## Continuation

Pursue refinements to the frequency-based winner (v001) by exploring tie-breaking strategies, counter saturation behavior, and computational costs at larger scales. The zone-based approach (v002) is a viable alternative if simplicity matters more than the marginal performance gain. Drop the ghost-list concept unless a fundamentally different insertion rule can be justified; the middle-position insertion was the wrong choice.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle refined the frequency-based eviction strategy from Cycle 0001 by exploring three different extensions: tie-breaking direction reversal (v001), zone-partitioning with hybrid frequency+LRU logic (v002), and periodic frequency decay (v003). The winning approach—periodic decay—introduces temporal adaptation, allowing the cache to gradually shed ancient hotspots and give weight to newly-popular items, while the zone-based hybrid approach demonstrated that combining multiple protection signals (recency for cold items, frequency for hot items) can also improve performance.

## Patterns

- Frequency-based protection improves hit rate, but static frequency counters have a weakness: without periodic reset, early hotspots retain dominance indefinitely. v003's modest but consistent gain from periodic decay (right-shift at capacity × 16 intervals) shows that workload phase shifts are present and exploitable.
- Hybrid zone-based eviction (v002) successfully combined the best of both signals: LRU eviction for one-time cold accesses and frequency-based selection for hot items that have proven reuse value. This two-tier approach captured real structure in the workload that a single-axis strategy misses.
- Tie-breaking direction within a frequency tier matters, but the original direction was correct: v001's attempt to prefer recency-ordered victims (tail-to-head scan) over age-ordered victims (head-to-tail) actually hurt performance, revealing that older low-frequency arrivals truly are colder and less likely to be re-accessed than newer ones. This refines the intuition that LRU is capturing real temporal locality.

## Open questions

- How does v003's performance compare to v002 in absolute terms? Both improved hit rate, but does the decay mechanism provide benefit over the zone-based hybrid, or do they capture complementary gains?
- Does the capacity × 16 decay interval generalize across different workload traces and cache sizes, or is it tuned to this specific scenario?
- Could the decay mechanism be combined with zone-based partitioning to achieve even stronger adaptation: decaying both cold and hot zone frequencies independently?

## Continuation

The frequency-based approach with periodic decay (v003) is the new winner and should become the baseline for Cycle 0003. Zone-based partitioning (v002) remains viable as a simpler alternative if the marginal gain from decay doesn't justify the added complexity. Abandon tie-breaking refinements along the recency axis; the head-to-tail (LRU) scan was correct and reversing it degraded performance. Explore whether decay thresholds can be tuned or made adaptive, whether combining decay with zone-based logic yields further gains, and whether other forms of workload adaptation (e.g., dynamic threshold adjustment) might capture phase shifts more efficiently than periodic halving.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle tested three refinements to the Cycle 0002 winner (periodic frequency decay with fixed `capacity * 16` interval): zone-based partitioning with independent decay rates (v001), adaptive frequency-distribution-aware decay thresholds (v002), and expanded hot-zone size (v003). All three regressed relative to the parent strategy. The "winner" v002 represents the least degradation, suggesting the baseline approach has already converged to a well-tuned point and that orthogonal modifications introduce overhead without corresponding benefit.

## Patterns

- **Zone partitioning consistently backfires.** Both v001 (cold/hot split with LRU in cold zone) and v003 (resized hot zone) hurt hit-rate. The parent's unified frequency-decay structure allows all items to compete fairly on frequency; introducing zone boundaries creates artificial promotion barriers and masks frequency signals that matter across the entire cache. Items with moderate frequency get evicted from the cold zone before demonstrating enough reuse to earn hot-zone protection.

- **Adaptive complexity underperforms fixed simplicity.** v002's attempt to dynamically adjust decay timing based on observed frequency distribution added tracking and per-operation computation overhead without yielding gains. The fixed `capacity * 16` interval—simpler and lighter-weight—remains superior, suggesting either this workload lacks sharp phase transitions requiring real-time adaptation, or the cost of tracking averages simply outweighs phase-shift benefits.

- **The parent strategy is robustly tuned.** The fact that all three orthogonal modifications regressed suggests the Cycle 0002 winner has already captured the major structural advantage (protecting hot items via frequency decay while gradually forgetting ancient hotspots). The narrow window for further improvement indicates this cache is near a local optimum for this workload.

## Open questions

- Is the workload fundamentally lacking the structural properties (e.g., temporal locality phase shifts, clear hot/cold bimodality) that would benefit from zone-based or adaptive strategies?
- Could a simpler variant—e.g., counter saturation caps, different decay patterns (exponential vs. linear right-shift), or within-frequency tie-breaking refinements—improve on the parent without introducing partitioning complexity?
- How sensitive is the `capacity * 16` interval to changes in cache capacity or workload scale? Is this interval portable across different scenarios?

## Continuation

The fixed-interval frequency-decay approach from Cycle 0002 remains the strongest baseline. **Abandon zone-based partitioning entirely**: all attempts (this cycle and Cycle 0002 v002) to split the cache by access pattern hurt performance, indicating the workload benefits from unified competition on frequency alone. **Stop pursuing adaptive decay mechanisms** unless a fundamentally lower-cost signal (e.g., hit-miss ratio trending, not frequency statistics) emerges. For Cycle 0004, explore simpler **within-frequency refinements**: counter saturation behavior (when do frequencies max out?), decay patterns other than right-shift, or tie-breaking rules when evicting items of equal frequency. These avoid partitioning overhead while probing whether the current strategy has exploitable margin.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle tested three refinements to the Cycle 0002 winner (periodic frequency decay at fixed `capacity * 16` interval): counter saturation capping (v001, neutral), more frequent decay at `capacity * 8` (v002, regressed), and smoothed multiplicative decay at 0.75× (v003, winner). The winning multiplicative approach reveals that the decay *curve shape* is more critical than decay frequency; the workload benefits from gentler frequency forgetting that preserves discrimination between hot and cold items across phase shifts.

## Patterns

- **Decay interval is robustly tuned.** Accelerating decay from `capacity * 16` to `capacity * 8` actively hurts hit-rate by forcing the cache to forget historical patterns too aggressively. This workload's phase shifts are gradual, not abrupt; the original interval balances frequency accumulation and adaptation correctly, and attempting to speed up adaptation backfires by destroying discriminative power before new patterns stabilize.

- **Decay curve shape outweighs frequency of decay.** The marginal gain from multiplicative decay (0.75× per cycle) over aggressive right-shift division (0.5×) demonstrates that smoother, less harsh forgetting preserves frequency discrimination longer. Items retain relatively higher frequencies across reset cycles, helping the cache reliably identify and protect truly hot items through workload transitions without losing track of historical patterns.

- **Counter saturation is not a bottleneck.** Capping frequency counters at 32 (vs. natural u8 255 saturation) has no measurable impact. The workload either does not generate frequency values near saturation, or the existing periodic decay mechanism refreshes priorities fast enough that an artificial lower cap adds no practical benefit.

## Open questions

- Does the 0.75× multiplicative decay coefficient generalize across workloads and cache sizes, or is this value tuned to this specific trace? How sensitive is the improvement to nearby coefficients (0.7×, 0.8×, 0.85×)?
- Could other decay patterns (exponential curves, piecewise linear decay, frequency-dependent decay rates) further improve hit-rate beyond the marginal gain observed here?
- Why does the right-shift decay (0.5×) lose discriminative power faster than multiplicative decay? Is this purely a function of the magnitude difference, or does the shape of the decay curve interact with the frequency distribution in non-obvious ways?

## Continuation

The multiplicative decay approach (v003) is the new winner and should become the baseline for Cycle 0005. The `capacity * 16` interval remains well-tuned; stop pursuing interval acceleration. **Explore decay curve variations** around the 0.75× multiplicative approach: test adjacent coefficients (0.7×, 0.8×, 0.85×) to see if there is a local optimum, and consider whether different decay patterns (e.g., linear decay, adaptive decay based on frequency distribution) might capture further gains. Abandon counter saturation as a lever; it is not constraining this workload. The current findings suggest the cache is tuning its "memory" via decay shape; the next cycle should probe whether this tuning window extends beyond multiplicative constants to other forgetting mechanisms.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle tested refinements to the Cycle 0004 winner (0.75× multiplicative frequency decay at `capacity * 16` intervals) by exploring coefficient sensitivity: increasing decay gentleness to 0.8× (v001), accelerating decay to 0.7× (v002), and replacing multiplicative decay with exponential approximation (v003). All three variants failed to improve upon the parent strategy; v002 and v003 actively regressed, while v001 showed no measurable benefit. The cache has reached a plateau around the current decay tuning.

## Patterns

- **The 0.75× decay coefficient is robustly tuned at a local optimum.** Adjustments in both directions—gentler (0.8×, approaching 1.0×) and more aggressive (0.7×, approaching 0.5×)—fail to yield improvements. V001's neutrality and V002's regression bracket the current value, indicating the workload's decay sensitivity converges at this specific coefficient, not as part of a broader continuum.

- **Exponential decay does not improve on multiplicative decay.** Although theoretically motivated by non-linear forgetting curves, the integer approximation (0.74 ≈ e^(-0.32)) introduces slightly more aggressiveness than 0.75× and undermines the frequency discrimination that the parent variant carefully preserved. The simpler linear multiplicative approach has already captured the optimal balance between forgetting and discrimination.

- **The cache is converging to a tight local optimum.** The failure of all three orthogonal perturbations suggests the current strategy has captured the major structural advantages available. Small changes to decay mechanics now hurt rather than help, implying the bottleneck for further improvement lies outside the decay mechanism itself.

## Open questions

- Is there a micro-tuned decay coefficient between 0.75× and 0.8× (e.g., 0.77×, 0.76×) that could extract marginal gains, or is the plateau independent of fine-grained coefficient adjustment?
- What aspect of the access pattern makes decay coefficient so critical? Is the workload's frequency distribution multimodal, causing sensitivity to forgetting rates, or are phase shifts the dominant factor?
- Could orthogonal decay mechanics—such as threshold-dependent decay (only decaying frequencies above a certain level), piecewise decay (different rates for hot vs. cold items), or frequency-dependent decay coefficients—circumvent the plateau without changing the baseline coefficient?

## Continuation

The 0.75× multiplicative decay approach from Cycle 0004 remains the strongest baseline and should be locked in as the foundation for Cycle 0006. **Abandon coefficient sensitivity testing**: both higher (0.8×) and lower (0.7×) values regress or stagnate, confirming the plateau is fundamental, not a missed optimization. **Do not pursue exponential or non-linear decay patterns** that approximate or replace multiplicative decay; the added complexity does not compensate for loss of frequency discrimination. For Cycle 0006, shift exploration away from decay mechanics entirely and probe orthogonal levers: test the `capacity * 16` reset interval for further optimization, examine whether frequency quantization or granularity improvements could sharpen discrimination, or investigate tie-breaking rules within equal-frequency items to improve hit-rate without modifying the decay mechanism itself.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle tested orthogonal levers outside the decay mechanism, following the plateau observed in Cycle 0005: extending the reset interval to capacity × 24 (v001), refining victim selection via two-pass LRU tie-breaking (v002), and selective decay for frequencies ≥ 4 (v003, rejected due to backend failure). v002 emerged as the winner, demonstrating that the cache still has margin for improvement in victim-selection *precision* rather than in decay mechanics. The baseline decay strategy (0.75× at capacity × 16) remains locked in as optimal, but orthogonal refinements can still yield measurable gains.

## Patterns

- **Reset interval is conclusively tuned.** v001's regression when extending capacity × 16 to capacity × 24 confirms (now for the second cycle) that this interval is robustly tuned. Deviations in both directions—accelerating (Cycle 0004, capacity × 8) or decelerating (this cycle, capacity × 24)—consistently regress. The decay mechanism itself is no longer a productive target.

- **Decay plateau is real; gains now come from orthogonal mechanisms.** After five cycles of tuning decay (coefficient, interval, curve shape), Cycle 0005's failure of all perturbations correctly identified a fundamental ceiling. v002's win this cycle confirms the next frontier is *victim selection precision*, not forgetting mechanics. The cache's frequency discrimination is already optimal; the bottleneck is ensuring the right item is evicted when frequencies are tied.

- **LRU ordering within frequency tiers is underutilized.** v002's two-pass scan—finding minimum frequency once, then selecting the head-most item with that frequency—captures temporal locality within equal-frequency groups more accurately than scan-order bias. This structural refinement respects the linked-list semantics and yields measurable improvement without any change to the decay or frequency-tracking code.

## Open questions

- How far can victim-selection refinement carry improvements? Are there other tie-breaking heuristics (e.g., recency-weighted frequency, decay-adjusted scores within frequency tiers) that could extend v002's gains?

- Does v003's selective decay (skipping decay for frequencies < 4) have merit, or was the backend failure fortunate? Threshold-dependent decay remains unexplored and could be a lower-cost orthogonal lever if the two-pass approach reaches saturation.

- How does v002's two-pass approach interact with potential future changes (e.g., counter saturation limits, frequency quantization, or different eviction policies)? Are there complementary refinements that compound with better tie-breaking?

## Continuation

Adopt v002's two-pass victim selection as the new baseline for Cycle 0007. The decay mechanism (0.75× multiplicative at capacity × 16 interval) is locked in and should not be revisited; this cycle definitively closes exploration of frequency decay. The win here signals the cache's optimization window has shifted from *forgetting mechanics* to *eviction precision*. For Cycle 0007, pursue further orthogonal levers: test alternative tie-breaking heuristics within equal-frequency items (e.g., frequency-recency hybrids), revisit v003's selective-decay hypothesis with a fresh implementation, or probe frequency-aware quantization to sharpen discrimination among hot items. The cache is converging toward structural completion; focus on refining decision-making at eviction time rather than the signals used to inform those decisions.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle tested three orthogonal approaches to extend the Cycle 0006 winner (two-pass LRU victim selection with 0.75× decay at capacity × 16 intervals): confirming the two-pass approach in isolation (v001, winner), attempting selective threshold-dependent decay to preserve weak frequency signals (v002, regressed), and introducing frequency-recency hybrid scoring as an alternative eviction paradigm tested against baseline (v003, showed promise on baseline). v001's win confirms the Cycle 0006 approach remains stable, while v002's failure definitively rules out selective decay, and v003 signals that hybrid scoring may merit deeper exploration when combined with the decay mechanism.

## Patterns

- **Two-pass victim selection is stable and confirmed.** v001's win, despite showing no measurable improvement over the parent, reflects that this refinement remains the best approach even as marginal gains shrink. The approach correctly captures temporal locality within equal-frequency tiers and has become the expected standard. The lack of measurable delta suggests victim-selection precision is no longer the main bottleneck; the decay mechanism has become so effective that tiebreaker quality no longer moves hit-rate significantly.

- **Selective decay uniformly fails; uniform decay is locked in.** v002's regression when preserving weak frequency signals (freq < 4) confirms Cycle 0005's finding that uniform decay is superior. Attempting to shield newly-accessed items from decay breaks the cache's ability to discriminate between ephemeral and persistent access patterns. The workload requires all items to decay proportionally; threshold-based exemptions allow weak-frequency items to remain artificially inflated and degrade eviction quality.

- **Frequency-recency hybrid scoring shows potential but was not tested against the decay incumbent.** v003 demonstrated clear improvement over baseline with a hybrid score combining frequency (×100 weight) and recency position. However, this fresh approach bypassed the proven decay mechanism entirely, leaving open whether hybrid scoring would compound with decay or conflict with it. The contrast between v003's help and v002's hurt suggests the workload may benefit from mixing frequency with other decision dimensions, but this requires validation within the decay framework.

## Open questions

- How does v003's frequency-recency hybrid scoring perform when combined with the 0.75× decay mechanism and two-pass LRU tie-breaking? Does hybrid scoring deliver additive gains on top of the incumbent, or does the decay mechanism already capture the benefit?

- Are victim-selection refinements now exhausted? v001's neutral delta despite being the winner suggests the cache may have reached the optimization ceiling for eviction precision. Should exploration pivot away from "how to evict" toward "what signals to use for eviction"?

- Does v003's hybrid score formula (freq × 100 + position) generalize, or is the 100× weighting tuned to this workload? Should coefficient sensitivity testing be revisited for the hybrid approach if it shows promise with decay?

## Continuation

Retain v001's two-pass victim selection as the baseline and lock in the full stack: 0.75× decay at capacity × 16 intervals plus two-pass LRU tie-breaking. Decay is conclusively optimized; stop revisiting threshold-based or alternative decay mechanics. **Prioritize investigation of v003's hybrid scoring**: test it in combination with the incumbent's decay mechanism to determine whether recency-weighted eviction scores deliver gains beyond frequency-only selection. If hybrid scoring shows measurable improvement when paired with decay, pursue coefficient optimization; if it remains neutral or regresses, abandon the approach and accept that the cache may have reached saturation on victim-selection improvements. The optimization window appears to be shifting from *eviction precision* back toward *eviction signal design*; the next cycle should clarify whether that window has opened or closed.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle tested the explicit open question from Cycle 0007: whether hybrid frequency-recency scoring would improve performance when combined with the proven decay mechanism. Three variants approached this through different parameter and architecture choices: preserving the two-pass selection with a 100× frequency coefficient (v001), using one-pass selection with the same coefficient (v002), and reducing frequency weighting to 25× while maintaining two-pass selection (v003). All three variants regressed, uniformly and across distinct implementation paths, conclusively demonstrating that hybrid frequency-recency scoring is incompatible with frequency decay.

## Patterns

- **Hybrid frequency-recency scoring fails universally with decay, independent of selection architecture or coefficient.** v001 and v002 tested the same scoring formula (freq × 100 + position) with two-pass and one-pass selection respectively, both regressing identically. v003's attempt to tune the coefficient downward (25×) also regressed, confirming the failure is not parameter mistuning but fundamental signal incompatibility. The workload's access patterns are sufficiently frequency-dominated that blending recency into the eviction score introduces noise that overrides gains from better tie-breaking.

- **Decay compresses frequency values, making position-based weighting dominant.** The 0.75× multiplicative decay at capacity × 16 intervals keeps all frequencies in a narrow range. When position_from_head (0–2047) is added to the score, this positional term dominates the compressed frequency signal, causing the cache to prioritize insertion recency over access frequency—inverting the decision boundary that the decay mechanism carefully calibrated. The two-pass approach's precision in finding minimum frequency becomes irrelevant if the scoring formula then overrides frequency discrimination with temporal bias.

- **The cache has converged to a local optimum where orthogonal mechanisms now regress.** After Cycle 0006–0007 exhausted victim-selection refinement (two-pass > one-pass, LRU tiebreaking), Cycle 0008 definitively closes the exploration of hybrid scoring. Every variant that deviated from the baseline (0.75× decay, capacity × 16 interval, two-pass LRU victim selection) regressed or showed no benefit. The optimization window has closed at this structural level; further gains—if any—must come from outside eviction decision-making.

## Open questions

- Is the cache fundamentally at saturation for this workload, having captured all available gains through frequency discrimination and LRU ordering? The universal failure of hybrid scoring after five cycles of decay tuning and two cycles of victim-selection refinement suggests structural completion may be the correct conclusion rather than a missed optimization.

- Does the workload's access pattern contain phase shifts or mode switching that could be exploited by adaptive decay or frequency quantization, or is the current strategy already capturing phase information implicitly through decay-driven frequency decay?

- Should exploration now pivot toward implementation-level micro-tuning (e.g., decay coefficient sensitivity between 0.74× and 0.76×, counter precision tradeoffs), entirely different eviction paradigms, or acceptance that the cache has reached its local optimum on this workload?

## Continuation

Lock in the Cycle 0006–0007 baseline permanently: 0.75× multiplicative decay at capacity × 16 intervals with two-pass LRU victim selection. Hybrid frequency-recency scoring has been conclusively ruled out across all architectures and parameter choices; do not revisit mixing recency or temporal bias into the eviction score. The workload requires pure frequency discrimination; orthogonal signals degrade performance.

For Cycle 0009, **accept that eviction signal and mechanism optimization may have reached saturation.** Exploration should pivot away from eviction decision-making toward either: (1) implementation-level optimizations (counter representation, decay triggering, data-structure efficiency), (2) entirely different eviction paradigms if this workload shows diminishing returns on frequency-based caching, or (3) fine-grained parameter micro-tuning if confidence in further structural gains is low. The evidence suggests the cache has converged to a tight local optimum; confirm this hypothesis before investing in marginal coefficient adjustments.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cycle tested three variations on the decay mechanism locked in by Cycle 0008: switching from periodic uniform decay to per-access activity-responsive decay (v001, regressed), quantizing 8-bit frequencies into 3-bit bucketed tiers (v002, neutral), and replacing fixed decay interval with adaptive hit-rate monitoring (v003, regressed). v002 emerges as the trivial winner by maintaining the baseline unchanged; all three variants either degraded performance or had zero measurable impact. This cycle confirms that decay mechanism refinement is exhausted: the 0.75× coefficient at capacity × 16 interval is definitively locked in, and no further parameter tuning, timing strategy, or precision trade-off will improve upon it.

## Patterns

- **Per-access decay fails by inverting the recency-frequency balance.** v001's discontinuous two-tier system (items accessed within 64 ops retain full frequency; older items decay sharply) recreates the exact failure mode of Cycle 0008's hybrid scoring: overweighting recency at the expense of cumulative frequency history. The uniform periodic decay mechanism has correctly calibrated the rate at which all items age together; making decay responsive to individual access timing breaks this global synchronization and privileges recent accesses over genuinely hot items. This is not a parameter-tuning failure but a fundamental incompatibility between activity-responsive and frequency-based eviction.

- **Frequency precision is irrelevant once decay compresses the range.** v002's neutral result confirms that the 0.75× decay compresses 8-bit counters into such a narrow operational range that both precise and quantized tracking converge to identical eviction behavior. This aligns with Cycle 0004's neutral finding on counter saturation; the workload's access patterns do not expose scenarios where fine-grained precision matters. Bucketing to 3 bits provides no benefit because coarse and fine granularity yield the same victim-selection ordering.

- **Adaptive mechanisms uniformly fail; fixed parameters have won.** v003's regression when adding hit-rate monitoring extends a multi-cycle pattern: Cycles 0005, 0007, 0008, and now 0009 have all attempted adaptive approaches (threshold-based decay, frequency quantization variance, hybrid scoring, and now hit-rate monitoring), and all have either failed or been neutral. The per-operation overhead of counters and periodic calculations does not justify the gains (none observed) from earlier decay triggering. The workload either lacks the phase transitions that adaptive mechanisms could exploit, or such transitions exist but are infrequent enough that fixed-interval decay already captures them.

- **Decay mechanism refinement is conclusively exhausted.** After Cycles 0006–0007 tuned the decay coefficient and interval, Cycle 0008 ruled out hybrid scoring with decay, and Cycle 0009 has now tested three orthogonal decay variations (timing model, precision, triggering strategy)—all have failed or regressed relative to the baseline. The cache has reached saturation at the decay level; no further variations on how, when, or with what precision decay occurs will yield improvement.

## Open questions

- Is saturation at the decay level equivalent to saturation across the entire eviction subsystem? Cycles 0006–0009 have exhausted decay tuning and victim-selection refinement; are there other orthogonal dimensions to explore (e.g., counter representation micro-tuning, cache sizing, or replacement of frequency-based eviction entirely)?

- Does the workload genuinely lack phase transitions, or are there adaptive strategies more sophisticated than hit-rate monitoring (e.g., entropy-based distribution shifts, frequency-distribution divergence) that could detect real transitions and justify the overhead?

- Should the next cycle accept structural saturation and focus on confirmation and micro-tuning, or pivot to entirely different optimization paradigms?

## Continuation

Lock in the Cycle 0008 baseline permanently with high confidence: 0.75× multiplicative decay at capacity × 16 intervals with two-pass LRU victim selection. Decay refinement is conclusively finished—per-access activity-responsive decay fails, frequency quantization provides no benefit, and adaptive interval triggering regresses. Stop all exploration of decay coefficient, interval, timing model, precision, and adaptation strategy. The mechanism is optimal for this workload.

For Cycle 0010, accept that eviction decision-making optimization has reached saturation. The cache has leveraged frequency discrimination, LRU ordering, and decay-driven phase adaptation to their limits. Future exploration must either: (1) pivot toward implementation-level micro-tuning (counter representation, decay triggering efficiency, data-structure layout) if such gains are available, (2) test entirely different eviction paradigms (e.g., non-frequency-based policies) to confirm frequency-based caching is the right strategy, or (3) formally declare saturation and close the optimization loop. The evidence strongly supports saturation; further cycles should confirm rather than assume.


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
