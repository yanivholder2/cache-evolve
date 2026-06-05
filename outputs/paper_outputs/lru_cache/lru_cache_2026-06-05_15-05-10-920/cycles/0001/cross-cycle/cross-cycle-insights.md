# Cross-Cycle Insights

## Recurring wins

- **Access-skew protection over recency alone**: Both high-performing variants (frequency-based v001 and zone-partitioning v002) converged on protecting frequently-accessed items from eviction, suggesting the workload exhibits access patterns where some items dominate usage and outlive temporal ordering.

## Recurring failures

- **Overly conservative re-insertion**: Ghost-list "second chance" (v003) positioned re-inserted items mid-list rather than prioritizing them, undercutting the core benefit of second-chance protection. The mechanism itself is sound; the insertion rule was the flaw.

## Open questions

- Why does frequency-based global counting narrowly edge out zone-based local thresholding? Is the margin driven by tie-breaking variance, cache locality, or fundamental structural difference?
- Can ghost-list re-insertion be salvaged by bumping insertion priority higher, or does coupling it with frequency tracking become necessary?
- How do memory overhead and eviction costs scale under these strategies as capacity grows?
- Which approach (global counters vs. local thresholds) degrades more gracefully under sustained load or shifting access patterns?

## Per-policy notes

- **Frequency-based (v001, winner)**: Global access counters protect hot items directly. ~9% hit rate gain. Warrants exploration of tie-breaking strategies, saturation behavior at large scales, and computational cost.
- **Zone-based partitioning (v002, strong performer)**: Local promotion thresholds segment the LRU into tiers. ~8.3% gain. Simpler than global counters if the marginal performance gap is acceptable; good fallback if frequency overhead becomes problematic.
- **Ghost-list "second chance" (v003, degraded)**: Re-insertion mechanism unsound due to middle-position placement. Core intuition of protecting wrongly-evicted items is valid; requires different insertion rule (tail or frequency-aware) to be viable.
