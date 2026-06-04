# Cycle 0002 Insight

## Cohort outcome

This was a follow-through cohort that took up cycle-1's three open threads
directly: frequency-driven retention (replacing the recency promotion
trigger), preserving-and-gating the v001 latency win behind a reuse
filter, and the metadata lever retried as a clean field-narrowing rather
than removal. Only the frequency-promotion variant held the front, and it
did so without demonstrably improving any axis.

## Patterns

- Two independent measurement-decoupling failures dominated the cohort,
  not policy logic. The frequency variant's hit-rate effect could not be
  attributed because the evaluated parent vector has no entry for the
  scan/loop trace it targets; the narrowing variant compiled and ran yet
  the metadata-size reading never moved, because the measured Item byte
  figure is sourced from somewhere other than the overlaid struct. The
  blocking factor on both the hit-rate and metadata axes is metric
  plumbing, not the change.
- Frequency as a signal is cheap and behavior-compatible (it reuses an
  existing per-item counter, so metadata held flat), but the chosen rule —
  strict pre-hit maximum with ties never promoting — is too conservative
  on a loop workload where residents share similar counts, so the hot tier
  stays under-filled. It also taxed get latency by swapping an O(1)
  positional check for an O(n) bucket scan.

## Open questions

- Where does the metadata-size number actually come from? Editing Item
  field widths in the overlay does not change it.
- How can hit-rate on the scan/loop trace be compared when the parent's
  fitness vector lacks an entry for that trace?
- The second-hit admission gate that would keep v001's O(1) back-push while
  filtering scan keys did not surface a result — was it run?

## Continuation

Before spending more attempts on retention or metadata, fix the
measurement path first: get the scan/loop trace into the evaluated vector
so hit-rate changes can be attributed, and locate the true source of the
metadata-size figure. Keep frequency as a promotion signal but drop the
strict-max/ties-never rule and the O(n) per-hit scan in favor of a cheap
threshold (e.g. second-hit admission) that preserves the O(1) promotion
path — this reuse-gate refinement of the front-runner is the most
promising live direction. Drop pure Item field-narrowing until the metric
source is found.
