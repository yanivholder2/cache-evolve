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
