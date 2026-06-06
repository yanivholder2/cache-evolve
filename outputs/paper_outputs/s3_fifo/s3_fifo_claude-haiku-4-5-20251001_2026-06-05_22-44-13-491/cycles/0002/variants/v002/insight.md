# Variant v002 Insight

## Hurt

- hit-rate: Assigning ghost hits an initial frequency of 1 creates artificial eviction resistance that does not reflect actual reuse patterns, causing cold items to be evicted prematurely and reducing overall hit rate.

## Takeaway

Ghost hits should not receive frequency credits upon re-admission. Despite being previously cached and accessed again, ghost items do not justify preferential protection through artificial frequency assignment—the baseline approach of re-admitting with frequency 0 allows the cache to better prioritize items with demonstrable access patterns. The heuristic that "prior survival implies future value" does not hold in practice.
