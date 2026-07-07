# Variant v001 Insight

## Helped

(None measured in this evaluation)

## Hurt

(None measured in this evaluation)

## Takeaway

Adding occupancy-aware selective insertion to filter low-value evictions from lightly-loaded buckets yields no improvement over unconditional stash admission. The 25% stash capacity from the parent already captures all meaningful working-set re-use; evicted items from under-occupied buckets contribute negligibly to hit rate because they are rarely re-accessed. The natural FIFO eviction within the stash discards them anyway, making proactive filtering redundant.
