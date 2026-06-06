# Cycle 0007 Insight

## Cohort outcome

All three variants forked from the cycle-0006 v001 trivial-hasher side-table
parent and tried to wring more latency from the `target_heat` probe: fuse the
hasher and capacity wins into one (v001), swap the HashMap for a direct-mapped
array (v002), and move the per-put heat-clear to the eviction site (v003). All
three entered the Winners, but the cohort overturned the cycle's framing —
every win landed on a different op than its hypothesis named, and the get
median proved untouchable.

## Patterns

- The composition was free but mis-aimed. Adding capacity pre-reservation on
  top of the trivial hasher (v001) regressed nothing and stayed bit-for-bit on
  hit-rate and metadata, but its tail win surfaced on the **put** path, not the
  get-p99 the disjoint-cause model predicted. The get path was already pinned
  at its hasher floor with no rehash spike left to remove, so the reservation's
  only headroom was the table-growth cost that put-side hits were absorbing.
- get-p50 is now a hard floor. Both v001 and v002 left the get median flat;
  v002's structurally cheaper direct-mapped array — meant to undercut the
  SwissTable group scan — did not move it at all. Cycle 0006's hasher swap had
  already removed the only median-path cost; what remains on the side-table
  probe is tail variance, not median latency.
- The structurally cheaper table still pays off as a variance reducer. v002's
  flat array dropped both p99 tails and put-p50 while holding hit-rate exactly,
  confirming 2×-capacity sizing makes slot collisions a negligible
  promotion-timing perturbation — a safe substitute even though it doesn't
  touch the median.
- The put-p99 tail is owned by eviction/cascade work, not the side-table
  touch. v003 confirmed this negatively: the per-put `remove` was the cheapest
  possible operation (a non-resident negative lookup that short-circuits), and
  relocating it to the eviction path turned it into a real resident remove
  behind new branches, regressing both put metrics.

## Open questions

- With get-p50 pinned and the side-table probe's median cost gone, is any
  get-path median lever left, or is get latency fully converged?
- v003 located the put-p99 tail in the eviction/cascade machinery. Is that
  cascade reachable without disturbing gate decisions or hit-rate, or is it the
  irreducible cost of the demotion structure?

## Continuation

Stop targeting get-p50 — two independent routes this cycle confirmed it is at
a hard hasher floor with no probe-side headroom left. Carry forward v001
(capacity reservation, a free put-tail win) and/or v002 (direct-mapped array,
tighter p99s) as the latency baseline; they are both safe, hit-rate-neutral.
The live frontier is now the **put path**, and v003 pinpointed its p99 bound as
the eviction/cascade work, not the side table — next cycle should probe that
cascade directly rather than economizing on the per-put probe, which is already
the cheapest form of its operation. Treat hit-rate as still fully capped.
