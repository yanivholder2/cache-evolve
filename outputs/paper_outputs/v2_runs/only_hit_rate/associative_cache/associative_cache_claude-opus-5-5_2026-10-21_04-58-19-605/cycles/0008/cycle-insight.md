# Cycle 0008 Insight

## Cohort outcome

All three variants tested a single knob on the cycle/0007/v001 CLOCK base (strict newcomer filter, 4-entry fingerprint ghost ring). v001 raised the first-hit promotion from 1 to 2. v002 recorded ghosts only for keys never hit while resident. v003 lowered the counter ceiling from 3 to 2. All three hypotheses were refuted, and each variant lost a small amount of hit rate. v002 is the listed winner only because its loss was the smallest. Its own insight says the ghost-admission filter gives no real leverage, so it adds no mechanism worth keeping.

## Patterns

- Hit-side protection now has a bracket on both sides. A bigger first-hit step (v001) and a lower ceiling (v003) both lost hit rate, so the uniform +1 step with a ceiling of 3 is locally optimal. Too much protection shields once-touched keys, and too little evicts keys that really are reused. Every GCLOCK knob has now been probed in both directions on workload-a.
- Ghost-ring admission filters lose on both lines, but for different reasons. On LRU (Cycle 0006), excluding never-hit keys hurt. On CLOCK, excluding hit keys hurt, though only barely: with the hand resting on the newcomer, most evictions are already never-hit keys, so the filter removes only a thin slice. Aged-out residents that were hit at some point are worth recording on both lines. The unfiltered 4-entry ring is already complete.
- One reward at the same small size works for both a ghost return and a first hit. Cycle 0004 found that the admission reward must be capped. This cycle finds the same cap applies on the hit side.
- This is the third cycle in a row with no hit-rate gain on workload-a, and the third in which the continuation asked for cost metrics that never arrived. v003's shorter worst-case sweep is a real cost saving that the evaluation still can't see.

## Open questions

- Would raising the ceiling *together with* a larger first-hit step help, so that the extra reward doesn't flatten the frequency range? It's untested, but the trend in this cohort suggests a small effect at best.
- Is ceiling 2 worth taking once per-op-time is measured? Its hit-rate loss is small and its worst-case sweep is a third shorter.
- Do any of the losses in the last three cycles exceed run-to-run noise on a second trace or cache size?

## Continuation

Treat cycle/0007/v001 as final for hit rate on workload-a. The design is fully bracketed: ring size, ring admission, newcomer grace, hit-step size and counter ceiling all peak at their current settings. Drop all single-knob CLOCK hit-rate probes, including ring-admission filters and hit-reward magnitude. The next cycle should not propose another hit-rate-only tweak on workload-a. It should get per-op-time and metadata-size recorded, or add a second trace or cache size, and then re-judge ceiling 2, a 3-entry fingerprint ring and the LRU+ghost reference line on those axes.
