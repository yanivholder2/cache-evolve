# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-06-05_15-05-10-920/cycles/0009

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-06-05_15-05-10-920/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-06-05_15-05-10-920/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-06-05_15-05-10-920/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-06-05_15-05-10-920/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-06-05_15-05-10-920/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-06-05_15-05-10-920/cycles/0009/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
