# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-opus-4-8_2026-06-06_02-45-01-395/cycles/0010

## Winners

- v002
- v003
- v001

## Variant insights

- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-opus-4-8_2026-06-06_02-45-01-395/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-opus-4-8_2026-06-06_02-45-01-395/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-opus-4-8_2026-06-06_02-45-01-395/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-opus-4-8_2026-06-06_02-45-01-395/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-opus-4-8_2026-06-06_02-45-01-395/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/paper_outputs/full_fitness_vector/dash/dash_claude-opus-4-8_2026-06-06_02-45-01-395/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants forked from the cycle-0007 incumbents and attempted
"behavior-preserving" structural moves on the dash segment: reorder the get
lookup branches stash-last (v001), relocate the per-put heat clear to the
demotion site (v002), and collapse the two-tier demotion cascade to a single
stash hop (v003). All three entered the Winners, but none was a clean win —
each move's premise was contradicted by where the work actually lives, and the
cohort overturned two standing assumptions at once.

## Patterns

- The get median lives in the **stash**, not the promoted tiers. v001 assumed
  the hottest hits resolve in target/probing and pay a wasted stash scan, so it
  moved stash last — and get-p50 regressed, because the median get hit actually
  resolves in the stash and now pays two failed scans before reaching it. The
  branch-ordering lever is real but points stash-*first*, the inverse of the
  hypothesis. Behavior was preserved exactly (hit-rate/metadata bit-for-bit);
  only the cost model was inverted.
- Relocating a probe off the median path trades median for tails. v002 did
  shave put-p50 by deleting the unconditional per-put heat clear, confirming
  that touch was real hot-path work — but the clear didn't vanish, it landed on
  the demotion branches and resurfaced as wider put-p99/get-p99 and a worse
  get-p50 summary. Moving work off the median is not free when the destination
  adds branching the tail then exposes — the same lesson cycle 0007's v003 taught
  on the eviction path.
- Hit-rate is **not** perfectly conflict-insensitive. v003 cut the put-p99
  cascade exactly as cycle 0007 predicted by collapsing the foreign-resident
  retry to a single stash hop — but hit-rate slipped below the parent, because
  the one-more-chance re-home was salvaging a small number of hits. This is the
  first crack in the multi-cycle "hit-rate fully capped / conflict-insensitive"
  framing: the demotion cascade is a genuine latency-vs-hit-rate trade, not pure
  overhead.

## Open questions

- If the get median resolves in the stash, is a stash-first reordering an
  actual get-p50 win, or does promoting the stash branch only relocate the cost
  onto the promoted-tier hits the way v002 relocated the put clear?
- The put-p99 cascade is now a priced trade: how large is the hit-rate cost of
  the single-hop demotion, and is there a cheaper conflict-mitigation step that
  keeps most of the rescued hits without the double-hop tail?

## Continuation

Two assumptions this cohort relied on were wrong and should be dropped: hot
get hits do **not** live in the promoted tiers (the median is in the stash, so
try stash-*first* if probing get-p50 at all), and the demotion cascade is **not**
free on hit-rate (collapsing it trades a few hits for the put-p99 win). The
put-p99/hit-rate trade (v003) and the put-p50/tail trade (v002) are both now
characterized as trades, not clean wins — next cycle should decide whether
either trade is worth banking rather than seeking a strictly-free version, and
should treat "hit-rate fully capped" as no longer exact in the conflict/demotion
machinery.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three variants forked from cycle-0007 incumbents and worked the demotion
cascade and get-path arithmetic that prior cycles had flagged: gate the
cascade retry on a non-full target (v001), defer the target/probing index math
past the stash branch (v002), and add a second rescue level to the cascade
(v003). All three entered the Winners, but none was clean — two were
characterized trades and one refuted its own premise, and together they
established that cascade depth is a continuous latency↔hit-rate dial coupled
to the get path through the stash.

## Patterns

- The cascade is a bidirectional dial, not a fixed cost. v001 shortened it
  (route full-target items straight to stash) and v003 deepened it (one more
  rescue level); the shallow direction sheds put-p99 and concedes hit-rate,
  the deep direction buys hit-rate and widens put-p99. v003 pushed hit-rate
  above the long-standing 0.6794 cap for the first time — but barely, at a
  steeply diminishing rate where each added level buys far less residency than
  the tail it widens. The cap is breakable but not cheaply.
- The stash is a shared resource that couples put-side cascade work to
  get-side latency. v001 revealed the parent's double-hop `put_at` was not pure
  waste: it kept the stash short at the get path's expense. Diverting
  full-target items into the stash to save the put tail lengthened the chain
  the stash-first get must scan, regressing get-p50/p99. Any cascade economy
  that ends in the stash pays at the get median — the two paths cannot be
  optimized in isolation.
- get-p50 is not arithmetic-bound. v002 deferred two confirmed-wasted
  `get_index` calls off the median stash-resolving path and get-p50 rose, not
  fell — the calls sat below the noise floor and the code motion only
  perturbed codegen. This closes cycle 0008's open question: the median get is
  hasher/probe-bound and fully converged; shaving "wasted" index math is not a
  lever.

## Open questions

- Both cascade directions ended in the stash and paid there. Is there a
  conflict-mitigation step that resolves victims *without* growing stash
  occupancy — so the put-tail win and the get-median cost decouple?
- v003 cleared the 0.6794 cap by a hair at large put-p99 cost. Is there any
  residency above the cap reachable at a tolerable tail, or is the cap
  effectively the practical ceiling given the diminishing return?

## Continuation

Three levers are now closed: get-p50 is hasher-bound (drop index-arithmetic
moves entirely), and both cascade directions are priced trades, not clean
wins. The load-bearing discovery for the next cycle is the **stash coupling** —
put-side cascade work and get-side median latency are linked through stash
occupancy, so any cascade change must be scored on both ops at once, not just
the one it targets. If pursuing hit-rate past 0.6794, expect to pay put-p99
steeply; if pursuing the put tail, expect get-p50 to absorb it via a longer
stash. Next cycle should look for a conflict-resolution path that does *not*
terminate in the stash, since that is the only way seen to break the trade
rather than slide along it.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
