# Cycle 0009 Insight

## Cohort outcome

All three probes ran off the cycle-0008 source-steered adaptive-split parent
and stayed on the new adaptivity axis, refining the controller three ways:
momentum + a wider clamp so the split accelerates and roams further (H01),
hysteresis gating that only moves after agreeing evidence accumulates (H02),
and a second independent controller that self-tunes ghost *capacity* by its
measured conversion productivity (H03). Only the momentum/wider-clamp variant
(v001) landed in the Winners; hysteresis was a marginal regression and the
adaptive-ghost controller regressed harder.

## Patterns

- The controller was under-reaching, not mis-aimed. Letting the split
  accelerate on sustained agreement and roam a wider band recovered hits the
  tight single-step orbit could not reach — confirming the cycle-0008 suspicion
  directly. But the gain stays small because this trace's imbalance drifts only
  modestly and monotonically: most of the widened headroom and the higher step
  cap go unused, so momentum mostly buys *faster convergence within a narrow
  effective range*, not a new operating point.
- The per-event steering "thrash" the parent suffered was largely real
  directional signal, not noise. Hysteresis (H02) throttled a controller
  already pointed the right way, adding lag without sharpening, and lost hits.
  This trace's slow, near-monotone imbalance wants a *lower* commitment
  threshold, not a wider accumulation band — denoising discipline is the wrong
  medicine when the signal isn't noisy.
- The feedback-controller framing does *not* extend cleanly to ghost capacity
  (H03). Pinning ghost capacity near `main_size` beats letting a loop trade it
  away, because the controller's floor undercut the parent's static ghost
  budget and a transient dip in windowed conversion rate discarded exactly the
  re-admission history the change meant to capture. Ghost history here is
  hit-productive enough that it should stay pinned, echoing the standing finding
  that ghost capacity is the binding, must-protect constraint.

## Open questions

- v001's momentum gain is real but capped by the trace's modest, monotone
  drift. Is there any remaining adaptivity headroom, or is the partition
  controller now effectively tuned for this workload?
- Two cycles of adaptivity (0008 split, 0009 momentum) have each paid only at
  the margin, and the second runtime knob (ghost capacity) regressed. Is the
  adaptivity axis now near exhaustion the way ordering and ghost-capacity were?

## Continuation

Adopt v001's momentum + wider-clamp adaptive split as the new parent — a small
but real gain that confirms the controller was under-reaching. Drop steering
hysteresis permanently (this trace's imbalance is monotone signal, not noise;
gating only adds lag) and drop adaptive ghost capacity permanently (regressed;
ghost capacity must stay pinned near `main_size` — its floor leaks productive
history). The adaptivity axis is looking close to tuned-out: the partition
controller now tracks this trace about as well as its modest drift allows, and
the one untried runtime knob backfired. The next cycle should either declare
convergence or find a lever on a mechanism not yet touched — not revisit static
sizing, ghost depth/capacity, ghost warm-start, weak-ghost removal, steering
hysteresis, eviction ordering, victim sharpening, or insertion position, all now
exhausted or falsified.
