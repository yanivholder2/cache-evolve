# Cycle 0008 Insight

## Cohort outcome

Off the cycle-0007 source-aware-ghost-retention parent, this cycle finally
left the now-exhausted ghost-capacity axis and probed three new surfaces: a
runtime-adaptive small/main split steered by the ghost `from_main` tag (H01),
dropping weak one-hit-wonder ghosts entirely to hand the full budget to strong
keys (H02), and recency-aware move-to-front in the thin small admission queue
(H03). Only the adaptive split (v001) landed in the Winners — the first
mechanism in many cycles to *beat* static sizing rather than collapse back into
it. H02 was a severe regression; H03 was inert.

## Patterns

- Adaptivity is a genuinely new lever and the standout result. Every prior
  sizing result came from static sweeps that converged on the frozen ~4/96
  ratio; reusing the already-computed `from_main` steering signal to nudge the
  eviction *threshold* (no entries moved) turns the ghost queue into a feedback
  controller that tracks the workload's shifting balance. The gain is small —
  the tight clamp keeps the orbit near the proven center and this trace wanders
  only modestly — but it moved the right direction and revealed the static
  split was slightly main-starved.
- The weak small-evicted ghost population, repeatedly called "low value,"
  is in fact load-bearing. H02 confirms from the extreme that those
  `from_main: false` records are the *dominant* re-admission source: deleting
  them collapses hit-rate by roughly a fifth. This sharply qualifies the
  cross-cycle finding — source-aware *trimming* (0007) pays precisely because
  it still ghosts weak keys; *not ghosting them at all* starves the far larger
  weak-reuse population. Reallocating budget to strong keys does not relax
  their constraint.
- The small queue's intra-ordering is too short a lever to matter. At ~4%
  capacity move-to-front only changes an outcome in the narrow tail-with-refill
  window, and it works against itself by delaying the `freq > 1` promotion into
  main, so the two effects roughly cancel. The binding constraint stays in main
  and ghost capacity, not small-queue recency.

## Open questions

- v001's adaptive gain is bounded by a deliberately tight clamp around the
  proven center. Is there headroom in widening the orbit, changing the step
  size, or using a smarter steering signal than a fixed ±1 — or does this
  trace's modest phase structure cap the payoff regardless?
- With both weak-ghost removal (H02) and small-queue ordering (H03) falsified,
  is adaptivity the last live axis, or does the feedback-controller framing
  open other runtime-adaptive knobs (e.g. adaptive ghost capacity)?

## Continuation

Adopt v001's source-steered adaptive partition as the new parent — the first
lever in several cycles to beat static sizing instead of collapsing into it.
Drop weak-ghost removal permanently (severe regression; weak `from_main: false`
records are the dominant re-admission source and must keep being ghosted) and
drop small-queue move-to-front permanently (inert; the queue is too thin and it
delays promotion). The live thread for the next cycle is the adaptivity
mechanism itself — tune the orbit/step or extend the feedback-controller idea
to another runtime knob. Do not revisit static sizing, ghost depth, ghost
warm-start, weak-ghost removal, eviction ordering, victim sharpening, or
insertion position — all now exhausted or falsified.
