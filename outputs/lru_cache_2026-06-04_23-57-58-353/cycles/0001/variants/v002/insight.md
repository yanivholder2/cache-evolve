# Variant v002 Insight

## Helped

- hit-rate: Replacing strict LRU eviction with CLOCK second-chance allowed entries that were accessed after insertion but before eviction pressure arrived to survive one extra clock revolution, reducing the chance of evicting still-warm entries on workload-a.

## Hurt

_(none)_

## Takeaway

For workload-a, the CLOCK second-chance reprieve is net positive: the workload contains enough entries that are accessed at least once after insertion that protecting them for one extra revolution outweighs the loss of strict recency ordering. The flat-arena circular scan also eliminates the linked-list pointer updates on every `get`, but the hit-rate gain comes from the second-chance policy itself rather than from the reduced per-operation overhead.
