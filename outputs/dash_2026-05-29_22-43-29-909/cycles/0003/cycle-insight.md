# Cycle 0003 Insight

## Cohort outcome

Three bets all aimed at the same goal — lift hit rate on the scan/loop
trace — by three different mechanisms: cycle-2's requested second-hit stash
admission gate, its requested cheap O(1) counter-threshold promotion, and a
brand-new lever, eviction-victim selection. Only the eviction-victim bet
held the front, landing as a recency-promote / frequency-evict hybrid that
kept metadata flat but cost get latency.

## Patterns

- The cohort opened a genuinely new axis: eviction-victim selection is
  independent of the promotion/position levers every prior cycle worked.
  The front-runner kept promotion strictly recency-based and changed only
  *who leaves* — choosing the least-reused resident — proving the victim
  slot is a separately tunable lever, and it stayed metadata-free by reusing
  the existing per-item counter.
- The same measurement-plumbing blocker flagged in cycle 2 is still
  unresolved and is now the binding constraint for a third straight cycle:
  every hypothesis targeted the scan/loop trace, but the parent's fitness
  vector still has no entry for it, so the entire hit-rate thesis behind the
  whole cohort remains unjudgeable. The front moved on latency/metadata
  defaults, not on the axis the cohort was actually chasing.
- The new lever bought its (unmeasurable) hit-rate intent with a latency
  tax — a per-hit counter write plus an O(n) full-bucket victim scan —
  echoing the cost structure of cycle-2's frequency promotion.

## Open questions

- When does the scan/loop trace finally enter the evaluated vector? Until it
  does, hit-rate-targeted bets cannot be attributed, and this has now
  blocked three cycles.
- The two promotion-path refinements cycle 2 called the most promising live
  direction did not surface as front results again — were they realized this
  cycle or did they fail to emit comparable output as before?
- Is the frequency-evict hybrid worth keeping while its hit-rate payoff
  cannot be measured, given it costs latency now for an unverified gain?

## Continuation

Fix the measurement path before spending more bets: get the scan/loop trace
into the evaluated vector so hit-rate is attributable — this is now a
prerequisite, not optional, because every cohort that ignores it just moves
the front by latency/metadata side effects. Keep the eviction-victim lever
as a real new direction but replace its O(n) victim scan with a cheaper
selection. The cheap-reuse-gate promotion refinement has now failed to
produce a comparable result twice; confirm it is actually being realized or
drop it.
