## Recurring wins

- Enlarging the main partition (shrinking the small/main admission filter)
  was the dominant lever through cycles 0001–0003, paying repeatedly until it
  plateaued near a thin admission filter; the workload tolerates a very thin
  filter without leaking one-hit-wonders.
- *Source quality* of ghosted keys is a live lever: ghosting the stronger
  main-evicted population recovered a dropped signal (0006), and dropping
  weak ghosts *before* strong ones under a fixed budget paid marginally
  (0007). The discriminating axis is the quality of the ghosted key, not the
  window length.
- Sharpening *which residents the eviction scan reaches* pays, while
  re-admitting or warm-starting weak keys does not. Recency-aware
  move-to-front on main paid a sliver (0004); the eviction-time rotation that
  refreshes survivor recency is load-bearing.
- Adaptivity beats static tuning: reusing the ghost `from_main` signal to
  nudge the partition threshold at runtime (0008) was the first mechanism in
  many cycles to *beat* static sizing rather than collapse back into it.

## Recurring failures

- Loosening admission/promotion thresholds or otherwise feeding more keys
  into main regresses — the main queue is contention-sensitive; protecting it
  beats feeding it.
- Ghost *depth* / wider history is falsified over both weak and strong
  populations: keys that don't re-hit inside the original window never convert.
  Capacity, not window length, is the binding ghost constraint.
- Ghost re-admission gating (small-queue routing, throttling, freshness) all
  regress; a ghost hit is itself reuse evidence and the direct-to-main path is
  correct.
- Frequency warm-start on re-admission regressed three separate times, over
  both populations — protecting weak-signal keys is wasted slack.
- Global frequency-aging sweeps over-decay (the rotation already does targeted
  decay); promotion-timing changes are inert under the thin partition;
  small-queue intra-ordering is too short a lever to matter.
- Removing weak `from_main: false` ghosts entirely is a severe regression —
  they are the *dominant* re-admission source, not low-value filler.
- Distinct-on-paper knobs (frequency ceiling, promotion threshold, promotion
  timing) repeatedly collapse into the partition-sizing knob once main is large.

## Open questions

- How much headroom remains in the adaptive partition: widen the orbit,
  change the step size, or use a smarter steering signal than a fixed clamp —
  or does this trace's modest phase structure cap the payoff?
- Does the feedback-controller framing extend to other runtime-adaptive knobs
  (e.g. adaptive ghost capacity), now that capacity is the binding ghost
  constraint that depth cannot relax?
- After many cycles yielding only sliver gains, is the configuration converged
  for practical purposes, or is adaptivity a genuinely new frontier?

## Per-policy notes

- All cycles tune one S3-FIFO design on a LIRS-style trace. The baseline's
  stricter filtering and its `evict_main` rotation (pop_back + push_front of
  surviving `freq > 0` entries) are load-bearing, not oversights.
- Exhausted/falsified axes: static partition sizing (frozen ~4/96), ghost
  depth, ghost re-admission gating, frequency warm-start, weak-ghost removal,
  eviction ordering, victim sharpening, insertion position, promotion timing,
  global aging, small-queue ordering.
- Live axes: source-aware ghost handling (quality + retention priority) and
  runtime adaptivity of the partition threshold — the latter is the most
  recent and most promising frontier.
