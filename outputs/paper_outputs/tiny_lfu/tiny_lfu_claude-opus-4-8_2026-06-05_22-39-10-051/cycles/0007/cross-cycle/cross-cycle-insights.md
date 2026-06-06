# Cross-Cycle Insights

## Recurring wins

- Spending the *existing* long-term frequency signal more carefully beats
  altering policy mechanics: the first clean gain came from frequency-ranked
  victim selection, and the most durable early lever was sharpening the
  estimator (wider count-min sketch) while leaving every mechanic untouched.
- Estimator fidelity was the single clearest lift on this trace — buying a
  more accurate count won where redesigning the signal's use lost.
- In later cycles, "wins" are won by *not regressing*: every structural lever
  (SLRU split, promotion gating, demotion choice) is directionally correct but
  clears the noise floor by only a hair.

## Recurring failures

- Any mechanism that discards, resets, ages, or withholds long-term frequency
  signal loses: sketch aging, unconditional admission window, doorkeeper.
- Extra admission filtering keeps failing — the bare `incoming_freq >
  victim_freq` gate plus a sharp sketch already screens one-hit wonders, so
  doorkeepers, entry floors, and admission-side placement only delay warming
  keys (or find nothing to gate) and cost hit-rate.
- Spending the signal more aggressively backfires: deeper victim samples
  plateau then reverse; stacking independent levers overlaps on the same rare
  cases and nets nothing.
- Aggressive memory narrowing (4x) breaks accuracy — the workload genuinely
  needs more than capacity×1 cells; conservative update does not rescue it.

## Open questions

- Is hit-rate at this capacity conclusively capped? Evidence across cycles
  0003–0007 is very strong — even a correctly-firing entry floor gates nothing.
- On the efficiency/cost axis (now the live frontier): does a *shallow* width
  reduction (e.g. 2x) hold accuracy where 4x failed?
- Are the marginal structural levers a workload ceiling rather than a tuning
  miss — would they pay off under a churnier or more skewed trace?

## Per-policy notes

- **LRU baseline / victim choice:** tail rarely hides a much-colder item;
  limited tail churn caps any victim-side change at a slim margin.
- **Sketch aging / admission window (W-TinyLFU):** inert on this
  popularity-stable, low-churn trace — they assume shifting popularity or burst
  protection that this workload does not exhibit.
- **Count-min sketch:** width is the key fidelity lever but plateaus once cells
  separate distinct keys; conservative update is redundant with a wide sketch
  and cannot hold accuracy at reduced width.
- **SLRU:** favors a *larger* protected tier (smaller-fraction line refuted);
  every steering point — admission, promotion entry, promotion gating,
  probationary eviction, protected demotion — has been mapped and each is
  marginal or redundant because recency order already tracks sketch frequency.
