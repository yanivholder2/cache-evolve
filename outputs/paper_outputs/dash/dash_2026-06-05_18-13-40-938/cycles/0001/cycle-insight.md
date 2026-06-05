# Cycle 0001 Insight

## Cohort outcome

This cycle tested three micro-optimizations to the LIRS-style tier hierarchy, all targeting the stash and its interactions with higher tiers. Two strategies improved hit rate by accelerating proven-hot items (stash residents) through the tier hierarchy; a third strategy testing frequency-based eviction showed no gain, confirming recency dominates this workload.

## Patterns

- **Stash hits merit fast-tracking**: Both hit-rate improvements (v001 and v003) prioritized stash-resident items for accelerated promotion, treating them as strong signals of interest worthy of immediate re-elevation toward the hot tier.
- **Positioning within a tier is a meaningful lever**: v001 achieved gains by changing insertion position from front (cold) to back (MRU) within the same target tier, suggesting that query order within a tier meaningfully impacts promotion eligibility.
- **Tier bypass trades traversal cost for placement priority**: v003 skipped the target tier entirely for stash hits, implying that elimination of intermediate steps can outweigh the benefit of tier-by-tier filtering.
- **Frequency-based eviction underperforms on recency-dominated workloads**: v002's LFU policy yielded no improvement despite correct implementation, validating that temporal locality (recent = reused soon) is a stronger predictor than access count on this traffic pattern.

## Open questions

- Why was v001 selected as the winner over v003, which also improved hit rate? (Magnitude of gain, tail behavior, or interaction with other subsystems?)
- Do v001 and v003 create synergy if combined, or do they interfere?
- Is there a sweet spot between fast-tracking and tier-by-tier filtering for stash hits?

## Continuation

Focus the next cycle on deepening stash-hit acceleration: both winning ideas converged on this axis, suggesting room for further refinement. Drop frequency-based eviction policies; this workload's recency-heavy access pattern makes them misaligned. Explore whether combining tier-position optimization (v001's lesson) with tier-skipping (v003's lesson) yields better results, or test variations on how stash-hitting items are weighted in promotion candidate selection.
