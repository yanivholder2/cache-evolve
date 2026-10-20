# Variant v002 Insight

## Helped

- hit-rate: Splitting the main region's single LRU into probation and protected segments means entries re-referenced while in probation are promoted and survive, while victims come from probation's LRU end, so one-off keys are evicted before entries that have been hit again.

## Hurt

## Takeaway

Changing only the ordering inside the main region (probation/protected, with the TinyLFU sketch and admission path untouched) produced a small hit-rate gain on workload-a, so the main region's own recency policy is a separate lever from the window experiment in cycle 0002. The signal rests on a single trace at one cache size, so it should be treated as provisional until other workloads confirm it.
