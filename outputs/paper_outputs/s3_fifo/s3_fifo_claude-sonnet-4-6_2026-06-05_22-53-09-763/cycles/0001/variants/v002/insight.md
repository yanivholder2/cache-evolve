# Variant v002 Insight

## Helped

*(none)*

## Hurt

- hit-rate: Promoting any once-accessed small-queue entry (freq > 0) into main flooded the main queue with borderline entries, displacing more genuinely hot items and causing them to be evicted before their next access.

## Takeaway

Lowering the promotion guard from `freq > 1` to `freq > 0` treats a single reuse in the small queue as sufficient signal to claim a main-queue slot, but for the workload-a trace this proves too permissive: single-access entries occupy space in main without generating hits, and main's second-chance mechanism isn't able to shed them fast enough to protect truly hot items. The baseline threshold of at least two accesses acts as a more reliable filter for entries worth retaining in the resident set.
