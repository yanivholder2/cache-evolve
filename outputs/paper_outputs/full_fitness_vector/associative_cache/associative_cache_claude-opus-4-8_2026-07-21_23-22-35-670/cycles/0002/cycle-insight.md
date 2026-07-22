# Cycle 0002 Insight

## Cohort outcome

This cycle recombined the two cycle-0001 winners along disjoint axes and
probed a policy refinement. The Winners were the clean cross-breed (v001,
stacking scan-resistant reuse-protection onto the slimmed item layout) and
a deeper metadata trim (v003, peeling one more dead field off the eval
path). The policy refinement — trading permanent protection for
second-chance aging — was the sole loser.

## Patterns

- The cycle-0001 winners composed additively with no interaction: the
  hit-rate mechanism (a reuse flag) and the metadata mechanism (layout
  slimming) touch disjoint code, and the reuse signal rides on a field the
  slim layout already keeps, so stacking them cost no extra bytes.
- Metadata trimming stayed monotonic and cheap one field deeper — dead
  weight carried only by off-eval-path policies keeps peeling off with zero
  hit-rate change, only tail-latency noise.
- On this trace, permanent reuse-protection is the feature, not a bug:
  giving protection a self-clearing second chance sheds immunity too fast
  and collapses back toward plain LRU. Cold-but-once-hot "squatters" are
  not what caps hit rate here.

## Open questions

- How much further can metadata shrink before it collides with a field the
  eval path actually reads — is the frontier near its floor?
- Since second-chance aging failed, is there any way to bound protection
  (cap how long a hot item stays immune) without shedding it too fast, or
  is permanent binary protection simply the right fit for this trace?
- The recurring upward p99 tail drift under flat p50 keeps appearing on
  metadata cuts — structural or persistent measurement noise?

## Continuation

Keep stacking the two proven, disjoint mechanisms and keep trimming dead
metadata off the eval path — both remain cheap and additive. Drop
second-chance / CLOCK-style aging on this trace; permanent protection wins,
so refinements should preserve it rather than weaken it. Next cycle should
test where the metadata floor is and whether any hit-rate headroom remains
beyond the now-composed reuse-protection win.
