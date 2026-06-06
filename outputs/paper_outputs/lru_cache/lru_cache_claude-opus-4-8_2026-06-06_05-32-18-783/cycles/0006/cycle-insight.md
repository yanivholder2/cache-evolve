# Cycle 0006 Insight

## Cohort outcome

This cycle ran two latency micro-optimizations against the 24-byte champion
and one geometry-sizing probe. Both micro-opts — eliding arena bounds
checks via unchecked indexing (v001) and fusing the unlink+push_back
relink to drop "dead" sentinel writes (v002) — held hit-rate and node size
but regressed per-op latency, advancing nothing. The lone advance was v003:
nudging the protected soft cap from 4/5 to 7/8 of capacity recovered
hit-rate above the parent with no latency penalty.

## Patterns

- Both latency micro-opts failed for the same reason and it is a cost-model
  error: the "redundant work" each removed was already free. Bounds checks
  on sequentially-predicted, proven-in-bounds arena accesses were elided or
  cheap (v001); the eliminated NIL stores were store-buffer-absorbed writes
  to an already-resident cache line (v002). In both cases the change traded
  a non-cost for real codegen damage — unsafe risk and register/code-size
  pressure from inlining — and came out net slower. The hot-path latency of
  this SLRU lives in hashing, link surgery, and cache-line traversal, not in
  the checks or writes these variants targeted.
- The Cycle 0004 lesson that protected-cap widening costs latency is now
  shown to be magnitude-dependent, not categorical. v003's modest one-eighth
  step bought hit-rate for free — get/put p50 even ticked down — whereas
  v003-from-Cycle-0004's larger share starved the fast path. There is real
  headroom between the parent's 4/5 geometry and the point where latency
  breaks down.

## Open questions

- How far can the protected fraction be pushed past 7/8 before the
  fast-path starvation seen at larger shares reappears — is there a clean
  cap maximizing hit-rate while latency holds?
- With the obvious micro-opt levers tapped, is the 27 ns champion latency
  near this trace's floor, or does a structural change (hashing, traversal
  layout) still have room the redundant-work edits never reached?

## Continuation

Promote v003's 7/8 protected cap as the new base — it is a free hit-rate
gain and the only frontier mover this cycle. Next cycle, probe the
remaining geometry headroom (a fraction between 7/8 and the starvation
point) rather than reaching straight for the largest share. Drop
bounds-check elision and the relink fusion as tried-and-regressed: both
attacked non-bottlenecks. If pursuing latency further, target the actual
hot-path costs (hashing, link surgery, cache-line traversal), not
compiler-handled redundancy.
