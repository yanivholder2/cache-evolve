# Variant v002 Insight

## Helped

- per-op-time: Put-side latency improved on both median and tail against the parent. Replacing backward-shift deletion with tombstones removes the shifting of following probe runs on delete, and sizing the table at about twice the live ghost cap keeps probe chains short on the put-miss path, which is where the parent's tail came from.

## Hurt

- metadata-size: The inline `S3Fifo` struct grew slightly against the parent, consistent with new bookkeeping fields (live and tombstone counts, table capacity) being added to the struct. The `Entry` size is unchanged.

## Takeaway

The hit rate matched the parent exactly, which confirms that tombstones and a larger table left the ghost's fingerprints and eviction order untouched. The put-tail gain supports H02's probe-work explanation, but this variant bundles tombstones and the lower load factor, so this result cannot say which of the two drove the improvement. The ghost's heap table footprint is still unmeasured, since the container-size probe does not see it; the next step is to report that footprint directly before judging the memory cost of the doubled table.
