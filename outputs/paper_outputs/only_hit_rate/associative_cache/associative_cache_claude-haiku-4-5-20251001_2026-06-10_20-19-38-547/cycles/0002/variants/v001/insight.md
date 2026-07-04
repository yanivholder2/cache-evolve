# Variant v001 Insight

## Helped

- **hit-rate**: LRU-ordered stash eviction preserves recently re-accessed items by tracking `last_accessed` timestamps, ensuring temporal working-set patterns stay in the stash longer than cold evictions.

## Hurt

(None measured in this evaluation)

## Takeaway

Switching from FIFO to LRU stash eviction is theoretically sound and correctly tracks access recency via timestamps, but delivers negligible improvement on the tested workload (0.6698→0.6699, ~0.015% gain). The FIFO stash was already effective at capturing second-chance hits; the workload's temporal patterns or stash hit frequency don't expose a meaningful advantage for LRU ordering.
