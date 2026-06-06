# Cycle 0005 Insight

## Cohort outcome

Off the converged ~4/96, move-to-front parent (cycle 0004 v003), this cycle
spent all three probes on main's ordering and eviction discipline:
recency-preserving in-place second-chance (H01), lowest-`freq` victim
selection within a tail window (H02), and tail-insertion of re-admitted and
promoted entries (H03). Every variant regressed; v001 and v002 "won" only as
the least-bad of an all-negative cohort. Nothing improved on the parent.

## Patterns

- The parent's `evict_main` rotation — `pop_back` plus `push_front` of
  surviving `freq > 0` entries — turned out to be load-bearing, not waste.
  Both H01 and H02 removed that rotation in favor of "in-place" or "more
  accurate" victim selection, and both regressed: the rotation was
  *refreshing* survivor recency and reinforcing v003's move-to-front
  ordering. Sharpening which victim the scan picks actively cost hit-rate
  because it discarded that refresh.
- The assumption that head-MRU purity is good and the tail is reliably
  coldest does not hold: `evict_small` and `put` also feed the head, so
  walking inward and decrementing every survivor (H01) over-erodes the `freq`
  signal of genuinely hot keys. The recency-vs-frequency tension is delicate
  and the parent already sits near its balance.
- H03 reconfirms the standing cross-cycle finding from the other direction:
  a ghost re-admission and a multi-hit promotion are strong-enough signals to
  deserve front protection on entry. Demoting them to the eviction-facing
  tail dropped them pre-reuse. The front position the parent grants these
  keys is earned, not waste.

## Open questions

- With both the victim-selection refinements and the insertion-position
  refinements regressing, is any remaining lever on the ordering axis live,
  or has v003's move-to-front already captured what the recency structure
  offers?
- Given that cycles 0003–0005 produced only sliver gains and now a fully
  negative cohort, is this configuration converged for practical purposes?

## Continuation

Treat the ordering axis as exhausted: the parent's rotation-on-eviction and
its front-insertion of re-admitted/promoted keys are both load-bearing — do
not disturb either. Every refinement this cycle (in-place decrement, windowed
victim selection, tail insertion) regressed, so drop all three. The
configuration appears converged; the next cycle should either declare it so
or find a lever on a mechanism not yet touched, not revisit eviction
ordering, victim sharpening, or insertion position.
