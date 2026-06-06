# Variant v003 Insight

## Helped

## Hurt

## Takeaway

Capping a promoted small-queue item's frequency to 1 before inserting it into main had no measurable effect on hit-rate for this workload. This indicates that items arriving in main with inflated small-queue freq credits are not, in practice, displacing genuinely hot main residents — either because the hot set is stable enough that promoted items re-earn their retention through subsequent main-queue accesses anyway, or because too few small-queue items accumulate a freq above 1 before being evicted for the normalization to change any eviction decisions. The hypothesis's concern about over-counted small-queue residents monopolising eviction reprieves does not appear to be the binding constraint on workload-a at this cache size.
