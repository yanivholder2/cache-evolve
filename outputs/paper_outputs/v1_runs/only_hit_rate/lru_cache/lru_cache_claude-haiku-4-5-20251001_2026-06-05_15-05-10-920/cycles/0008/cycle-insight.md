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
