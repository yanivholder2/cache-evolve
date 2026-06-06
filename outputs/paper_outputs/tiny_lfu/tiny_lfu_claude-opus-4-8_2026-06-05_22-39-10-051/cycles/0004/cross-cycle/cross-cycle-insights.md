# Cross-Cycle Insights

## Recurring wins

- Sharpening the frequency *estimator* (wider count-min sketch) is the single
  clearest lever seen so far — it won outright once, beating every policy-design
  change in the same cohort.
- Spending the existing frequency signal *carefully* — ranking a few LRU-tail
  candidates by estimate for victim selection — produced the only early
  hit-rate lift, though a slim one.
- In later cycles "winners" increasingly win by *not regressing* rather than by
  lifting hit-rate; the durable lever has consistently been upstream count
  fidelity, not the selection or admission rule.

## Recurring failures

- Any mechanism that discards, resets, or withholds long-term frequency signal
  fails: aggressive sketch aging, an unconditional W-TinyLFU admission window,
  and doorkeepers all lost. The trace is popularity-stable and low-churn, so
  burst/shift protection solves a problem that isn't present.
- Extra admission filtering is refuted across multiple cycles. The bare
  `incoming_freq > victim_freq` gate already screens one-hit-wonders, so added
  doorkeepers only delay warming keys and cost hit-rate.
- Spending the frequency signal more *aggressively* backfires: deepening the
  victim sample plateaus then reverses, trampling recency-warm / warming-hot
  keys before their counts accumulate.
- Stacking accuracy moves is redundant, not additive: width and conservative
  update land where each lands alone once cells already separate distinct keys.

## Open questions

- Is the marginality of the structural levers a *tuning* issue (larger sample,
  smaller protected fraction) or a genuine *workload ceiling*?
- Would victim sampling / SLRU pay off materially only on a churnier, more
  skewed workload?
- Is there any remaining lever on this trace that clears the noise floor, or is
  hit-rate at this capacity effectively capped?
- Could a deliberately *narrower* sketch + conservative update hold accuracy at
  lower memory cost — shifting the search to efficiency rather than hit-rate?

## Per-policy notes

- Count-min estimator: width is the main proven lever but plateaus fast after
  one big step (cells stop colliding with distinct-key count). Conservative
  update is redundant with a wide sketch — proven by direct combination.
- Frequency-ranked victim sampling: small sweet spot; directionally positive
  but reverses when widened; independent of estimator accuracy.
- SLRU (protected/probationary split): independent of accuracy, directionally
  positive, but nearly inert against the current admission gate.
- Admission window / doorkeeper: inert-to-harmful on this workload; no burst or
  popularity shift for them to exploit. Treat as parked/refused.
- LRU baseline + freq gate: already keeps the hot working set resident, which
  is why most structural rescues hover at the noise floor.
