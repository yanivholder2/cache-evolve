# Variant v002 Insight

## Helped

## Hurt

- hit-rate: floating `prot_target` within the [0.90, 0.98] band off the parent's
  fixed floor(0.95) boundary, steered by ghost-hit feedback, landed slightly
  below the static 95/5 split — the adaptive geometry drifted to a worse
  partition than the constant it replaced.

## Takeaway

H02's prediction is falsified on workload-a: ghost-driven resizing did not beat
the static 95/5 boundary, it regressed hit-rate marginally. The two nudge signals
are not symmetric in effect here — a returning probationary one-hitter
(`nudge_target_down`, grow probationary) and a returning demoted-then-evicted key
(`nudge_target_up`, grow protected) fire at different rates, and the net drift
moved the boundary away from 95/5 rather than toward a better split. Combined with
the parent's finding that the *probationary floor size* is the binding constraint,
the up-nudges that shrink that floor are the likely culprit: the cache cannot
discover a better-than-static split from reuse-distance feedback alone on this
trace, so 95/5 remains the closed optimum and the ghost machinery is pure overhead
to hit-rate.
