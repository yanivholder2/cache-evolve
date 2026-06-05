# Cycle 0007 Insight

## Cohort outcome

This cycle attacked three distinct, previously-untested levers against
the converged 95/5 SLRU incumbent: a third recency-of-Nth-access "hot"
tier inside protected (H01), an adaptive ghost-fed boundary that floats
the protected/probationary split at runtime (H02), and a widened LIRS
non-resident horizon (H03). All three regressed. v002 (adaptive
boundary) landed in the Winners only as the least-regressive change,
winning by confirming a negative.

## Patterns

- The 95/5 split is not just the best *static* point — it resists being
  improved *adaptively*. Letting the boundary float on reuse-distance
  feedback drifted to a worse partition than the constant it replaced,
  because the two nudge signals fire asymmetrically and the up-nudges
  erode the very probationary floor that all prior cycles identified as
  the binding constraint. The cache cannot discover a better split from
  reuse feedback alone here.
- Stratifying protected by reuse *depth* (LRU-K-like, not frequency)
  also fails: a depth-split protected segment starves its cold tier and
  churns out the twice-seen working set before it earns a third hit. A
  single flat protected LRU retains the reuse-heavy set better — reuse
  depth is not the binding signal on this trace.
- LIRS still has no demonstrated headroom. Widening the non-resident
  horizon surfaces exactly the long-distance, high-IRR second touches
  that should stay unprotected, and the unconditional non-resident-to-LIR
  promotion turns the extra memory into LIR pollution. The harm is in
  admission policy, not horizon length, so the capacity-bound horizon was
  already at or past optimum.

## Open questions

- Every off-axis lever now tried — depth stratification, adaptive
  geometry, LIRS horizon — has regressed. Is there any remaining
  structural axis on this trace, or is the incumbent a true converged
  optimum?
- The long-standing question from cycles 0005-0006 — does an *untuned*
  LIRS parent actually beat the 95/5 SLRU incumbent on a clean re-run —
  remains unresolved; this cycle only tuned LIRS (and failed) rather than
  verifying the parent.

## Continuation

Treat SLRU geometry as definitively closed: static is exhausted at 95/5
and adaptive resizing is now also disconfirmed — do not probe boundary
tuning again. Depth-stratified protected tiers and widened LIRS
non-resident memory are both dead; drop them. The single piece of
unfinished business is still the clean untuned-LIRS-vs-incumbent
verification that cycles 0005-0006 called for and no cycle has executed.
The next cycle should either run exactly that verification or, if it
again declines, declare the cohort converged on the 95/5 SLRU optimum —
three consecutive cycles of off-axis probes have now all regressed.
