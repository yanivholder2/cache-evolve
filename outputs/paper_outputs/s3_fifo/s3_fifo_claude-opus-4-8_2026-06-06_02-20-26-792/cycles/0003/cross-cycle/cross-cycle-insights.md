## Recurring wins

- Enlarging the main (resident) queue via the small/main partition ratio is
  the one lever that paid off repeatedly — across cycles it carried the
  sweep from the baseline through 10/90, ~6/94, to ~4/96.
- Each successful step came from giving the main queue more *capacity*, not
  from loosening what enters it; protecting main consistently beat feeding it.

## Recurring failures

- Gating the ghost re-admission path — via small-queue routing, throttling,
  or freshness/recency — regressed every time it was tried. A ghost hit is
  itself confirmed reuse evidence; any gate discards that signal faster than
  it earns anything back. The direct-to-main re-admission path is load-bearing.
- Stacking more retention onto an already-large main buys almost nothing:
  raising the frequency/second-chance ceiling and tightening the
  small-to-main promotion gate both came out inert once main was large.
  These distinct-on-paper knobs collapse into the partition-sizing knob.
- Loosening the admission/promotion threshold to admit more keys (early
  cycles) crowded the main queue and cost more than it saved.

## Open questions

- The partition-ratio sweep has flattened at ~4/96 (the knee); should this be
  frozen as the converged operating point, or is there residual headroom?
- Is there any lever genuinely *orthogonal* to main-queue retention left?
  Untried-on-its-own candidates: ghost queue *sizing* without a re-admission
  gate, and the second-chance/frequency mechanics themselves.
- Does the workload's remaining headroom justify continued tuning at all, or
  should effort shift to a different mechanism entirely?

## Per-policy notes

- S3-FIFO (LIRS workload): the baseline's stricter admission filtering is a
  feature, not an oversight — moving toward the paper's looser defaults
  (promote-on-any-reuse, deeper ghost history) regressed. The partition ratio
  is the dominant and near-exhausted knob; promotion threshold and ghost
  re-admission gating are inert or harmful beneath a thin admission filter.
