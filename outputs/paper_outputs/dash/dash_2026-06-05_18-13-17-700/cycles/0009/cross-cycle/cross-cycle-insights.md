## Recurring wins

- Reallocating fixed budget toward the promotion-protected normal tier was the
  earliest and largest win (cycles 0001–0002); shrinking the over-provisioned
  stash repeatedly lifted hit-rate until the split saturated around the
  normal-dominant ratio.
- The stash's load-bearing value is *dwell time* — giving new/cold keys room to
  prove a second reference. Levers that extend dwell (relocation hop, bounded
  multi-hop cascade) yield clean positives where promotion-side levers do not.
- Demotion/eviction-path insert position is a genuinely independent lever from
  the hit-path position: landing demoted or probing-evicted keys at MRU
  protected the oscillating working set and gained after hit-path tuning flattened.
- Two-speed promotion (slow stash→target, fast target→probing) reserves the
  protected tier for twice-seen keys and gains where uniform promotion stalled.
- The strongest late principle: *which population a lever spares matters more
  than its shape*. Orthogonal, warm-population seams stack on the winner —
  a ghost gate for dropped-then-recurring keys (LIRS short-reuse) and a
  breadth-of-homes hop on the warm probing tier both netted positive, while the
  same breadth idea was inert on cold stash overflow.

## Recurring failures

- Removing or gutting stash admission regresses sharply (cycles 0004, 0005) —
  the admission filter is protective, not dead probation; front-landing cold
  keys makes them instant eviction victims and is closed in both directions.
- Promotion-gate widening is occupancy-bottlenecked and now exhausted across
  every signal shape — quarter-gate, half-gate, density-independent touch
  counter, and len-2 loosening all returned slivers or slight negatives. The
  ceiling is the trace's small (len-1/len-2) target buckets, not the rule.
- Stacking independently-discovered seams overlaps rather than compounds when
  they chase the same boundary-key pool already reclaimed by promotion (single
  hop onto the winner, power-of-two admission, deeper/wider cascade on the
  winner all washed out or inverted).
- Geometry trades under conserved budget are neutral-to-negative: widening
  buckets while halving count regressed (collisions), and the symmetric
  width-for-count trade was exactly flat — addressability and depth are
  interchangeable at fixed capacity.
- Changing how the protected tier *ranks* residents (LFU eviction) regressed —
  recency/promote-on-hit already fits the skew; LFU retained stale-frequent keys.

## Open questions

- Do the two latest orthogonal positives (ghost gate, probing-tier breadth)
  stack with each other, or overlap once both ride the winner?
- Can the "spare warm victims, not cold ones" principle revive other inert
  stash ideas when re-pointed at the warm probing/protected tiers?
- Is non-additivity and the occupancy ceiling a property of this single
  high-skew trace, or general? A second, less-skewed trace has been overdue for
  validation since cycle 0004 and is now the highest-value next step.

## Per-policy notes

- Recency / promote-on-hit on the protected (normal) tier is the consistent
  best fit for this skewed trace; promotion-position and two-speed promotion
  refine it marginally.
- LFU eviction on the protected tier is confirmed dead here.
- Stash (cold admission): recency-with-dwell is essential; MRU default for
  cold keys holds, front-insertion regresses. Relocation/cascade helps on a
  clean baseline but inverts once the protected tier is present (it evicts
  already-warmed residents).
- Ghost-list assist (LIRS-style) on dropped-then-recurring keys is the one
  population promotion never recaptures and the freshest orthogonal win.
