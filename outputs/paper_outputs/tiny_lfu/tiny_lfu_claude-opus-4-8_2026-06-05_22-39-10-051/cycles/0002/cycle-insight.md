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
