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
