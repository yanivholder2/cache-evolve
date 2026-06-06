# Cross-Cycle Insights

## Recurring wins

- Buying a sharper estimator was the single cleanest lift on this workload:
  widening the count-min sketch raised hit-rate where no policy-mechanic
  change could. It was the one durable upstream lever.
- Spending the *existing* frequency signal carefully — ranking a few tail
  candidates by estimate — won the first cycle, where mechanisms that
  discarded signal lost. Careful selection beat aggressive selection.
- Structural levers (frequency-ranked victim sampling, SLRU
  recency-protection, frequency-gated promotion, protected-tier demotion)
  are consistently *directionally* correct and avoid regressions, even when
  their lift is at the noise floor.

## Recurring failures

- Any mechanism that resets, withholds, or bypasses long-term frequency
  signal loses: aggressive sketch aging, an unconditional admission window,
  and doorkeepers all cost hit-rate. This is the most robust negative result.
- Extra admission filtering is refuted across multiple cycles. The bare
  `incoming_freq > victim_freq` gate already screens one-hit-wonders, so
  added doorkeepers only delay warming keys.
- Spending the signal more aggressively backfires (deeper victim samples,
  shrinking the protected fraction) — it tramples recency-warm or
  warming-hot keys before counts accumulate.
- Pushing an already-exhausted lever adds nothing: further sketch width and
  conservative update proved redundant with each other once cells separated
  the distinct-key set.
- Compounding overlapping mechanisms does not stack — victim sampling and
  promotion-gating fire on the same rare case and net to roughly nothing.
- Null results have twice traced to mechanism bugs (an unsatisfiable entry
  floor); verify a lever actually engages before reading it as workload
  evidence.

## Open questions

- Is hit-rate at this capacity now conclusively capped? Evidence across the
  later cycles strongly suggests a ceiling, with every structural placement
  mapped and each marginal.
- Could a *narrower* sketch paired with conservative update hold accuracy at
  lower memory cost — trading now-wasted width for contention CU can correct?
- Would the marginal structural levers ever pay off materially under a
  churnier / more skewed workload, or is their marginality intrinsic here?
- When should the search decisively pivot from hit-rate to efficiency/cost
  dimensions instead of further structural refinement?

## Per-policy notes

- **Workload signature:** popularity-stable, low-churn. Mechanisms assuming
  shifting popularity (aging) or burst protection (window) find no problem to
  solve and only add noise. A lever matters only when the LRU head hides a
  key much colder or hotter than recency order implies — a rare event.
- **Estimator (count-min sketch):** the productive axis, but now closed.
  Width lift plateaus once cells separate distinct keys; width and
  conservative update are redundant, not additive (confirmed by direct test).
- **Admission gate:** the `incoming_freq > victim_freq` test plus a sharp
  sketch already keep the hot set resident, which caps every downstream
  structural lever. Added admission filtering / doorkeepers refuted.
- **SLRU:** comprehensively mapped — admission, promotion entry, promotion
  gating, probationary eviction, and protected demotion all probed. Each is
  real but marginal or redundant. The protected tier should stay *large*;
  shrinking it is refuted. Recency order within both segments already tracks
  sketch frequency closely, so frequency contests rarely reverse a decision.
