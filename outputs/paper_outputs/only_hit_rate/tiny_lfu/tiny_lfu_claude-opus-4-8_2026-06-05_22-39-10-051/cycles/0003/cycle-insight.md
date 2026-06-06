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
