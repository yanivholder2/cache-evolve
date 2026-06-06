# Cycle 0006 Insight

## Cohort outcome

Forked from cycle 0005's promotion-gate winner, all three variants stayed
on the SLRU axis and probed where else the sketch signal could steer
movement: stacking victim sampling onto the promotion gate (v001), applying
sampling to the protected-tier *exit* / demotion choice (v002), and adding a
floor threshold on protected entry (v003). v002 took the Winner slot on a
hair-thin lift; v001 added nothing over its parent, and v003 was an
inadvertent no-op.

## Patterns

- The compounding bet from cycle 0005 is now answered and refuted: victim
  sampling and frequency-gated promotion do *not* stack. Both mechanisms fire
  on the same scarce "head hides a hotter key" events, so layering them
  overlaps rather than adds — confirming they target one shared rare case,
  not disjoint ones.
- The one new placement for the sketch signal — the protected-tier *exit*
  (which node to demote on overflow) — is directionally right and edged
  out the parent, but only because it occasionally spares a frequency-hot,
  recency-stale resident. Its room to move is tiny: protected recency order
  already tracks sketch frequency closely, the same workload signature seen
  across cycles 0003–0005.
- A "lever" that never actually engages produces exactly parent results. The
  entry-floor variant was inert not because the idea is wrong but because
  increment ordering made the gate unsatisfiable — a reminder that null
  results need a mechanism check before they count as evidence about the
  workload.

## Open questions

- Every distinct point where the sketch can steer SLRU movement — admission,
  promotion entry, promotion gating, probationary eviction, protected
  demotion — has now been touched and each is marginal-or-inert. Is there any
  structural placement left untested, or is the hit-rate axis fully mapped?
- The exit lever (v002) won this cycle; would a stronger entry gate (a floor
  that actually fires, e.g. evaluated pre-increment) reserve protected slots
  in a way the current always-true entry does not?

## Continuation

The SLRU hit-rate axis is now comprehensively mapped: admission, promotion,
victim choice, and demotion have all been probed and every lever is real but
marginal or redundant. The compounding hypothesis is refuted — sampling and
promotion-gating overlap, do not stack. Across cycles 0003–0006 the evidence
that this workload's hit-rate sits at its ceiling is now very strong. Next
cycle should stop chasing hit-rate through structural refinement and pivot
decisively to efficiency/cost dimensions; if any hit-rate probe is attempted,
make it a corrected entry-floor (one that can actually fire), not another
victim/demotion sampler.
