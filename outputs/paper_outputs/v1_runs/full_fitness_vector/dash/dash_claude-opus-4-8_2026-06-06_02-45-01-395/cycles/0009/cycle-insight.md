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
