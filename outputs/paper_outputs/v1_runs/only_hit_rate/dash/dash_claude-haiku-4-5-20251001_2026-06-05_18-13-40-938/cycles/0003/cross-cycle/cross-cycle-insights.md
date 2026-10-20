# Cross-Cycle Insights

## Recurring wins

- **MRU (back insertion) positioning is a universal lever**: All three cycles converged on back insertion as a consistent win across promotion, demotion, and tier advancement. The effect holds regardless of the tier transition context—stash-hit promotion (0002), target-tier advancement (0003), or demotion cascades (0003).
- **Fitness signals gate acceleration**: Stash-resident items and items with advancement history consistently improved hit rate when fast-tracked. Occupancy alone, without a fitness check, is insufficient and creates churn.
- **Stash-hit acceleration is robust across implementation paths**: Cycles 0001–0002 validated the core insight from multiple angles—deterministic tier-skip (v001) and occupancy-adaptive skip (v002 in 0002) both improved hit rate, confirming the mechanism itself, not the exact implementation, matters.

## Recurring failures

- **Frequency-based eviction underperforms on recency-dominant workloads**: Cycle 0001 conclusively showed LFU provides no gain; temporal locality is a stronger signal than access count in this traffic pattern.
- **Occupancy-adaptive branching adds complexity without benefit**: Cycle 0003's v001 combined adaptive destination selection with MRU positioning yet hurt hit rate compared to deterministic baselines. Conditional occupancy logic is noise; consistent positioning rules work better.
- **Occupancy relief without fitness checks creates churn**: Cycle 0002's v003 (promoting any target item when stash > 60%) failed, while occupancy-gated promotion of stash hits (0002 v002) succeeded. The threshold level matters less than the fitness filter.

## Open questions

- Why were v001 alternatives (v002 or v003) not selected as winners despite similar or matching hit-rate gains? Magnitude, tail behavior, or synergistic interactions with other subsystems?
- Do successful variants from different cycles (e.g., 0002's v001 and 0003's v003) combine additively when stacked, or do they saturate a single underlying mechanism?
- Does the MRU back insertion principle extend beyond promotion and demotion to intra-tier operations and other insertion contexts?
- Under sustained high load / stash congestion, does the deterministic stash-hit skip (0002 v001) degrade while occupancy-adaptive strategies (0002 v002) excel, or are they functionally equivalent?

## Per-policy notes

- **MRU-based positioning** (back insertion): Consistently successful across cycles; treat as a production-ready baseline. Apply uniformly to promotion, demotion, and advancement paths.
- **Occupancy-adaptive strategies**: Fragile when used for destination selection or unfiltered promotion; more robust when gating accelerated promotion of fitness-validated items (stash hits). Keep gates tight and fitness checks first.
- **Frequency-weighted approaches**: Not validated; recency-only promotion rules are sufficient for this workload.
- **Tier-by-tier progression with deterministic positioning**: Preferred over conditional tier-skipping in later cycles (0003) when tested against adaptive branching. Simplicity and consistency outperform occupancy-aware destination logic.
