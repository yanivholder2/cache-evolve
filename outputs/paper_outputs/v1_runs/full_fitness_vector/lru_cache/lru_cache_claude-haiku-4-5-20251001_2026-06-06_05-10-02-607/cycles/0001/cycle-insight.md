# Cycle 0001 Insight

## Cohort outcome

Three strategies targeting hit rate improvements via different eviction signals were tested: spatial zone partitioning (V001), frequency-recency hybrid (V002), and resurrection count tracking (V003). V002 achieved the strongest hit rate and get latency gains, while V001 improved put performance through simpler eviction decisions. V003 showed modest hit rate gains but with higher latency variance.

## Patterns

- **Hit rate-latency trade-off**: Both V002 and V003 improved hit rate by making eviction decisions more selective, but this selectivity required O(N) worst-case scans through the eviction list, causing dramatic tail latency spikes (put p99: 20× for V002, 2× for V003). V001 avoided this by pre-partitioning but saw no hit rate benefit.

- **Metadata cost is universal**: All three variants added tracking overhead—zone markers, frequency counters, or eviction history maps. This increases per-item or per-cache memory footprint consistently, a tax paid even by underutilized caches.

- **Get and put performance diverge by strategy**: V002's frequency tracking improved both get latency (faster spatial locality from retaining hot items) and put median (less churn), yet spiked put tail. V003 hurt both get and put tails through metadata contention. V001 improved put but penalized get with promotion logic.

- **Eviction search efficiency is the bottleneck**: V002 and V003 both suffer from linear scans to find an evictable node when most candidates have protective marks (high frequency or resurrection count). The simpler zone-based eviction in V001 avoided this, suggesting that structural partitioning may be more latency-predictable than selective scanning.

## Open questions

- Can hit rate improvements be achieved without full linear eviction scans? (e.g., bucketed frequency lists, approximate resurrection tracking)
- Does V002's frequency decay mechanism, if better tuned, actually suppress the observed put p99 spike in production workloads?
- How sensitive are V002 and V003 to cache saturation and working-set stability? Do their metadata costs exceed benefits on underutilized caches?

## Continuation

The cycle reveals a fundamental tension: selective eviction improves hit rate but invites latency unpredictability, while structural partitioning is latency-friendly but may miss hit rate gains without per-item intelligence. V002 stands out for improving hit rate *and* get performance; its put p99 spike merits targeted mitigation (e.g., eviction bucket structures or heuristic pruning) rather than abandonment. V003's marginal gains do not justify its overhead. Next cycle should pursue either: (a) efficiency optimizations for V002's eviction scan (approximate frequency or bucketed lists), (b) V002 frequency decay tuning to empirically validate tail latency suppression, or (c) hybrid approaches that combine spatial robustness (V001) with selective intelligence (V002) under latency constraints.
