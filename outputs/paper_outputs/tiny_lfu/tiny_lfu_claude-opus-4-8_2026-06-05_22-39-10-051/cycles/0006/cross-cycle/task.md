# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants forked from the baseline along canonical TinyLFU axes —
sketch aging (v001), a W-TinyLFU admission window (v002), and frequency-aware
victim selection (v003). Only v003 landed in the Winners, with a small but
clean hit-rate gain and no regressions.

## Patterns

- The two variants that discarded or bypassed long-term frequency signal —
  aggressive sketch decay and an unconditional admission window — both lost
  hit-rate, while the one that *spent* the existing signal more carefully
  (ranking a few tail candidates by estimate) won.
- This workload appears popularity-stable and low-churn: mechanisms that
  assume shifting popularity (aging) or burst protection (window) found no
  problem to solve, so they only added noise.
- Wins here are slim. v003's gain is bounded by how rarely the LRU tail hides
  a much-colder item than the head, suggesting the trace has limited tail
  churn for any victim-side change to exploit.

## Open questions

- Are the aging and window mechanisms inert because of *parameterization*
  (10x-capacity threshold too aggressive, 1%-capacity window too small) or
  because the workload genuinely lacks churn? v001/v002 takeaways blame the
  former; v003's slim margin hints at the latter.
- Does v003's effect grow with a larger sample size, or does it plateau fast?

## Continuation

Build on v003: it is the live Winner and the only change that improved fitness.
Probe whether a larger frequency-ranked sample extends the gain before
concluding the workload caps it. Treat sketch aging and the admission window as
parked, not dead — they are likely mis-sized rather than wrong, so revisit them
only as parameter sweeps (less aggressive reset threshold, larger window
fraction) or if a churnier workload is introduced. Do not re-spend cycles on
mechanisms that discard the baseline's long-term frequency signal.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Three variants all attacked the frequency signal — widening v003's victim
sample (v001), adding a doorkeeper to filter one-hit wonders (v002), and
quadrupling the count-min sketch width (v003). Only v003 won: sharpening the
estimator while leaving every policy mechanic untouched was the sole change
that raised hit-rate.

## Patterns

- This workload is bounded by estimator *noise*, not policy *design*. The
  one variant that simply bought a more accurate count won; the two that
  altered how the signal is spent or gated both lost.
- Spending the frequency signal more aggressively backfires. Both the deeper
  victim sample and the doorkeeper ended up discarding recency-warm or
  warming-hot keys before their counts accumulated — recency encoded in LRU
  position and freshly-admitted-key signal are easy to trample.
- Last cycle's slim victim-ranking win was already near its sweet spot, not
  the start of a trend: it plateaus and then reverses when widened. The
  durable lever turned out to be upstream, in count fidelity, not in the
  selection rule.

## Open questions

- Does sketch-width fidelity keep paying off if pushed further, or does it
  too plateau once cells stop colliding with the trace's distinct-key count?
- Are the doorkeeper and deep-sample losses purely about resetting/over-
  spending fresh-key signal, and could an admission-only doorkeeper (no
  victim-ranking interference, less frequent clearing) still pay off?

## Continuation

Pursue estimator fidelity — v003's wider sketch is the live Winner and the
clearest lever on this workload. Sweep sketch width (and depth) to find where
the accuracy gain plateaus before spending cycles elsewhere. Drop the deeper
victim sample: it is refuted, plateaus then reverses. Keep the doorkeeper
parked but not dead — only revisit it restricted to admission with a much
longer clear window, never gating the main counters. Continue avoiding any
mechanism that resets or withholds long-term frequency signal.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants forked from cycle 0002's wide-sketch winner and stayed on
the estimator-fidelity axis: another 4x sketch-width step (v001), conservative-
update increments (v002), and an isolated admission-only doorkeeper with a long
clear window (v003). v001 and v002 landed in the Winners, but both only matched
the parent within noise; the doorkeeper (v003) regressed and is refuted.

## Patterns

- The estimator-fidelity lever has flattened. Once the count-min was widened in
  cycle 0002, every further accuracy move — more width, or conservative update —
  bought essentially nothing, because cells already separate the trace's
  distinct keys. Width and CU turned out to be redundant with each other, not
  additive, on this workload.
- Both "Winners" this cycle won by *not regressing*, not by lifting hit-rate.
  The real signal is that the accuracy axis has been exhausted: there are no
  more collisions left to remove.
- Extra admission filtering is the wrong direction, again. The bare
  `incoming_freq > victim_freq` test already screens one-hit-wonders, so even a
  carefully isolated long-window doorkeeper only delayed warming keys and cost
  hit-rate — consistent with cycle 0002's doorkeeper loss.

## Open questions

- With both the policy-design and estimator-accuracy axes now mined out, is
  there any remaining lever on this trace, or is hit-rate at this capacity
  effectively capped?
- Would conservative update finally pay off only if paired with a deliberately
  *narrower* sketch (trading the now-wasted width memory for contention CU can
  correct) — i.e. is there a width/CU combination cheaper than the parent at
  equal accuracy?

## Continuation

Stop sweeping estimator fidelity: width has plateaued and conservative update
adds nothing on top of the wide sketch — both are now parked as exhausted. The
admission-doorkeeper line is fully refuted across two cycles; drop it for good.
Next cycle should change axis entirely rather than refine accuracy: probe
whether a narrower-sketch + conservative-update tradeoff can hold accuracy at
lower memory, or treat this workload's hit-rate as near its ceiling and shift
the search toward efficiency/cost dimensions instead of raw hit-rate.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle finally changed axes off estimator fidelity: it directly
combined width×CU (v001), layered frequency-ranked victim sampling onto the
accurate sketch (v002), and split the main space into a protected/probationary
SLRU (v003). v003 took the Winner slot, but only on a hair-thin lift — every
new lever this cycle proved real-but-marginal rather than transformative.

## Patterns

- The accuracy axis is now closed by *direct* test, not inference: building
  width and conservative-update together landed exactly where each landed
  alone, confirming the cycle-0003 redundancy claim that had only been
  inferred from separate variants.
- The two structural levers — victim sampling and SLRU recency-protection —
  are each independent of estimator accuracy and each directionally positive,
  yet both pay almost nothing here. The shared cause is the admission gate:
  the `incoming_freq > victim_freq` test plus a sharp sketch already keep the
  hot working set resident, so there is little left for either mechanism to
  rescue.
- Recurring signature of this workload: a mechanism only matters when the LRU
  head hides a key much colder, or much hotter, than the gate already detects.
  That situation is rare on this trace, which is why every structural win
  hovers at the noise floor.

## Open questions

- Would victim sampling or SLRU pay off materially only under a larger sample,
  a smaller protected fraction, or a more skewed/churnier workload — i.e. is
  the marginality a tuning issue or a workload ceiling?
- With accuracy closed and two structural levers each nearly inert, is there
  any mechanism left on *this* trace that clears noise, or is hit-rate at this
  capacity effectively capped?

## Continuation

Treat the estimator-accuracy axis as definitively closed — width×CU
redundancy is now proven, not inferred; do not revisit. The structural levers
(SAMPLE victim scan, SLRU) are confirmed independent and directionally correct
but marginal against the current admission gate; keep them only as the live
substrate and probe whether parameter tuning (larger sample, smaller protected
fraction) or a churnier workload unlocks them — do not expect lift from them
unchanged. Strongly consider that this workload's hit-rate is near its ceiling
and shift the search toward efficiency/cost dimensions rather than chasing
further hit-rate.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants forked from cycle 0004's SLRU winner and refined the
split rather than changing axes: frequency-ranked victim sampling inside
the probationary tier (v001), a frequency contest gating protected-tier
promotion (v002), and shrinking the protected cap to two-thirds (v003).
v002 took the Winner slot on a hair-thin lift; v001 also nudged just past
the parent, while v003 regressed and is refuted.

## Patterns

- Spending the sharp sketch on *which key leaves or stays within the SLRU
  segments* is directionally right in both forms tested — victim choice
  (v001) and promotion gating (v002) — but each clears the noise floor by
  only a hair. The shared cause is unchanged from prior cycles: the
  admission gate plus accurate sketch already keep the hot set resident, so
  these refinements only fire on the rare case where the LRU head hides a
  key much colder, or much hotter, than recency order implies.
- The protected-fraction tuning question from cycle 0004 is now answered:
  the SLRU favours a *larger* protected tier, not a smaller one. Enlarging
  probation traded away protection without buying admission accuracy, so
  the marginal SLRU effect is a workload ceiling, not a downward-tuning
  miss.
- Recency order within both the protected and probationary segments already
  correlates closely with sketch frequency on this trace, which is why the
  frequency contests so rarely reverse a promotion or eviction.

## Open questions

- With victim choice, promotion gating, and protected sizing all explored
  and each marginal, is there any structural lever left on this trace that
  clears noise, or is hit-rate at this capacity now conclusively capped?
- Do the two confirmed-positive refinements (v001 sampling, v002 promotion
  gate) compound if combined, or do they overlap on the same rare cases and
  net to nothing?

## Continuation

Treat the SLRU structural axis as essentially exhausted for hit-rate:
victim sampling and frequency-gated promotion are confirmed real but
marginal, and the protected fraction is best left large — the smaller-
fraction line is refuted, do not revisit. The accumulated evidence across
cycles 0003–0005 strongly indicates this workload's hit-rate is at its
ceiling. Next cycle should either test whether the two positive refinements
compound (a cheap, final hit-rate probe) or decisively pivot the search to
efficiency/cost dimensions rather than chasing further hit-rate.


### Cycle 0006

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


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
