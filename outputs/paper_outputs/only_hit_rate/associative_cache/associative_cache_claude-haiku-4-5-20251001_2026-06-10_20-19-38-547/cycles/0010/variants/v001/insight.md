# Variant v001 Insight

## Helped

(None measured in this evaluation)

## Hurt

(None measured in this evaluation)

## Takeaway

Extending age-tracked LRU eviction from the hot tier to the cold tier yields no measurable improvement in hit-rate for this workload. This suggests the two-tier mechanism is already capturing temporal locality patterns effectively through hot-tier prioritization and promotion-on-hit logic alone. The cold tier's secondary role as a stash buffer may be too small or see too infrequent re-access to benefit from fine-grained LRU ordering; FIFO eviction within cold storage is sufficient when the hot tier is doing the real work of retaining frequently accessed items.
