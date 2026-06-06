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
