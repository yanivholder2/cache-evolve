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
