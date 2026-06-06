# Cross-Cycle Insights

## Recurring wins

- **Spatial locality via selective eviction**: V002's frequency-recency tracking improved hit rate and get latency by retaining hot items longer—demonstrating that per-item intelligence can improve access patterns.
- **Structural simplicity for put performance**: V001's zone-based partitioning reduced put latency variance by eliminating complex eviction decision logic.

## Recurring failures

- **Linear scan eviction penalty**: Both V002 and V003 suffered dramatic tail latency spikes (V002 put p99 20× baseline, V003 2×) when eviction requires scanning protected items; selectivity creates latency unpredictability.
- **Metadata overhead on underutilized caches**: All three strategies added per-item or per-cache tracking (frequency counters, zone markers, resurrection maps) that taxes memory even when the cache is not saturated. Overhead was universal across variants.
- **Marginal gains don't justify complexity**: V003's modest hit rate improvements could not justify higher latency variance and metadata cost, establishing a threshold below which selective tracking is economically irrational.

## Open questions

- Can selective eviction (V002-style) be made latency-predictable without linear scans? (Bucketed frequency lists, approximate resurrection tracking, or heuristic pruning.)
- Does V002's frequency decay mechanism, when properly tuned, actually suppress put p99 spikes in real production workloads, or is the spike inherent to linear searching?
- How does cache saturation affect metadata overhead? Do V002/V003 cost savings exceed their tracking burden only above a saturation threshold?

## Per-policy notes

- **V001 (spatial zone partitioning)**: Cleanest latency profile, improved put performance via pre-structured partitions. Trade-off: no hit rate gain despite simpler design. Suitable for latency-sensitive, read-heavy workloads without hit rate pressure.
- **V002 (frequency-recency hybrid)**: Strongest hit rate and get latency improvements; frequency tracking creates spatial locality. Critical issue: put tail latency requires targeted mitigation (e.g., bucketed eviction lists or heuristic candidate selection) to become production-ready.
- **V003 (resurrection count tracking)**: Adds metadata complexity without commensurate latency or hit rate win; metadata contention raised latency variance across both get and put. Low priority for further iteration unless resurrection patterns prove predictive in specific workloads.
