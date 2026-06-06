# Cycle 0008 Insight

## Cohort outcome

All three variants branched off cycle 0007's O(1)-get hash-index incumbent
and split their fire across the two remaining live targets: the last linear
scan on the *put* path (the ghost membership walk) and the long-frozen
metadata-size dimension. Two variants eliminated the put scan by different
mechanisms; one finally moved metadata-size off its seven-cycle plateau.
All three were Winners, hit-rate held bit-identical.

## Patterns

- The ghost membership scan was the binding put cost, now confirmed by
  removal. Both put-latency variants crushed put p50/p99 by replacing the
  O(ghost_len) `iter().any` + `retain` rebuild with an O(1) probe — one via
  a dedicated ghost HashSet, one by folding ghost state into the existing
  `entries` map as value-less tombstones. This was the last linear walk on
  any hot path; the resident scan was already gone in 0007.
- The two put fixes embody opposite memory tradeoffs for the *same*
  latency win. The separate set duplicates every ghost key (a second
  key-keyed table); the folded-tombstone variant adds no table but taxes
  the get path with an enum-discriminant match and inflates the resident
  map with ghost tombstones. The scan elimination is settled; how to pay
  for it is the new contested axis.
- Metadata-size finally moved. The `key` field — dead weight ever since
  0007's hash index made the resident key recoverable from the map key —
  was deleted, collapsing the scored layout below its frozen plateau. A
  delayed dividend of the prior cycle's index, and a free win that touched
  no lookup, eviction, or promotion path.
- Container memory is now the explicitly contested frontier the prior
  cycles flagged. Every put variant grew the unscored `S3Fifo` container
  (ghost set, ghost counter, or tombstone-inflated map) even as the scored
  per-entry cost shrank.

## Open questions

- Of the two O(1)-ghost designs, which is net-better once unscored
  container memory is weighed — the duplicated-key HashSet, or the
  tombstone-laden single map that taxes get?
- With both hot-path scans now eliminated and the scored entry shrunk, is
  any latency or memory lever left, or is workload-a at a joint hit-rate /
  median-latency / scored-size floor?
- Do the tombstone designs accumulate stale order-queue / map entries that
  degrade under longer or different inflow than tested here?

## Continuation

The scan frontier is closed: get (0007) and now put (0008) are both O(1),
so stop hunting hot-path linear walks — there are none left. The two
remaining live threads are both about *cost of the same hits*, not more
hits: (1) reconcile the two ghost-O(1) designs against unscored container
memory, since they make opposite tradeoffs for an identical latency win,
and (2) keep probing the now-unfrozen metadata-size dimension for further
dead-field deletions the hash index may have made redundant. Treat
hit-rate as closed (bit-identical at the same value for many cycles
running). Expect the next binding constraint to be container/tombstone
memory, not latency.
