# Variant v003 Insight

## Helped

- hit-rate: Replacing IRR with staleness (`op_clock − last_access`) as the secondary eviction signal improved eviction quality for entries seen only once — IRR returned `u64::MAX` for all such entries (making them indistinguishable), while staleness gives each one a real, monotonically increasing value that correctly ranks cold single-access entries against each other within the scan window.

## Hurt

## Takeaway

IRR's blind spot is entries that have been accessed exactly once: every such entry carries `u64::MAX` regardless of how long ago that access was, collapsing the tie-breaker for the most common cold-entry population. Staleness has no such blind spot — it is finite and growing from the very first access — so it can rank newly arrived entries against stale-burst entries at the same frequency level. The small but consistent gain on workload-a confirms that this population (single-access entries competing at `freq=0`) is common enough to matter, and that a tie-breaker which actually ranks them is worth more than one that treats them all as equally cold.
