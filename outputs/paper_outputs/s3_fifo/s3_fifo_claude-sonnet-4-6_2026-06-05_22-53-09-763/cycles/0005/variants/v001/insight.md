# Variant v001 Insight

## Helped

- hit-rate: Raising `MAX_FREQ` from 4 to 5 gave the hottest main-queue items one additional decrement step before eviction, retaining a thin tail of items whose access frequency genuinely reaches the prior ceiling and would otherwise have been evicted a pass too early.

## Hurt

## Takeaway

The 4→5 ceiling bump produced a smaller gain than the 3→4 step did in cycle/0004, confirming that the workload's hot-set tail thins out rapidly above the prior ceiling rather than extending densely. The extra second-chance pass does help a non-zero slice of items, but the population is so sparse that the return is nearly negligible — the frequency-ceiling axis is now effectively exhausted and further increments are not a productive direction.
