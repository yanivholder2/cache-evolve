# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Per-access decay based on operation deltas since each node's last access creates uneven frequency aging across the cache. Heavily accessed items never decay while rarely accessed items decay sharply once the 64-operation threshold is exceeded, leading to premature eviction of moderately-accessed items and worse hit-rate compared to uniform periodic decay.

## Takeaway

Switching from uniform periodic frequency decay to activity-responsive per-access decay significantly harmed cache performance. The per-access approach creates a discontinuous decay model where nodes accessed within the last 64 operations retain their frequencies unchanged, while older nodes suddenly experience aggressive decay. This breaks the balance that periodic decay (0.75× every 64 operations) achieved: instead of all items aging uniformly to adapt to phase shifts, this variant creates a two-tier system where recency is overweighted relative to frequency history, causing the cache to lose discriminative power and evict items that were genuinely hot but happened to fall outside the 64-operation window.
