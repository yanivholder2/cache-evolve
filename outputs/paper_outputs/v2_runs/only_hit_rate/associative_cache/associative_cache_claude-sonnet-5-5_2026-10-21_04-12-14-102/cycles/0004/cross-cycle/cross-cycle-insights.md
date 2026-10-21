## Recurring wins

- Structural separation of one-touch from re-referenced entries (probation/protected split) is the one idea that has paid off repeatedly. It gave the largest single gain (cycle 2).
- Shrinking the probation share keeps helping, but with diminishing returns. Independent routes in cycle 4 both ended at a small probation segment. The turning point has not clearly been found.
- A lean split on baseline geometry appears to capture most of the benefit. Extra bookkeeping added in earlier cycles looks unnecessary.
- Wider buckets gave a small, early hit-rate gain. It is the base geometry for later work, but its cost is unmeasured.

## Recurring failures

- Weakening or delaying hit-path promotion (transposition, midpoint, half-promotion, two-hit rule) lowers hit rate. First-hit move-to-back is hard to beat, especially with short or small-probation segments.
- Ghost-history ideas (ghost rings, ghost-hit promotion, adaptive targets) added nothing over the plain split. A ghost hit is weak reuse evidence here.
- Small-window frequency victim selection at the LRU end changed nothing measurable.
- Placement outside the hashed bucket (two-choice) fails the slot-legality invariant. This holds for any multi-bucket scheme unless the gate is relaxed.
- Per-op-time and metadata-size have been missing from the fitness vector in every cycle. All wins are hit-rate only, and every added piece of state is unpriced. Comparisons against plain LRU instead of the current parent also overstate increments.

## Open questions

- Where is the minimum probation size? Do sizes below the current point still help, or does scan resistance break? Check one size above as a control.
- Does the lean split match the full accumulated stack, so older state can be removed?
- Do hit-rate gains survive once per-op-time and metadata-size are measured? Fixing the evaluation is overdue.
- Would a larger probation segment make delayed promotion viable, or is first-hit promotion simply better?
- Would ghost history work with a stricter second signal, or should it be dropped?

## Per-policy notes

- Plain move-to-back LRU buckets: the reference. Promotion tweaks lose to it. Geometry changes give small gains.
- Probation/protected split (2Q-like): the winning family. Small fixed probation with first-hit promotion is the best known configuration.
- Frequency-aware victim choice: no effect at small windows. A wider window or a real probation-to-protected frequency test is untried.
- Ghost/adaptive variants: no gain over the fixed split. Avoid unless promotion is made stricter.
- Two-choice or multi-bucket placement: blocked by the invariant.
