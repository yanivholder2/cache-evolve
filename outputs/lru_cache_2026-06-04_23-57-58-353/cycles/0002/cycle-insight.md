# Cycle 0002 Insight

## Cohort outcome

The cohort probed three directions off the cycle-1 SLRU winner: loosening the segment-size ratio, layering frequency-aware demotion inside protected, and adding a ghost admission set. Only the ghost-admission variant (v003) won, by combining two-segment SLRU with cross-residency warmth tracking.

## Patterns

- The 80/20 protected-to-probationary split has now been stress-tested from both sides: cycle-1 confirmed it beats plain LRU, and this cycle confirmed that widening probationary to 30% hurts — the hot working set is large enough that the smaller protected segment triggers more demotion churn than the extra scan-filtering headroom saves.
- Frequency-aware demotion inside protected produced no measurable effect when the scan window covers only a small fraction of the segment and the frequency counter resets to zero on every demotion. The signal evaporates before it can accumulate durable discrimination between high- and low-reuse entries.
- Ghost admission targets a different gap than in-cache frequency: keys evicted from probationary that re-arrive before the ghost window expires have effectively earned two accesses across residency periods and deserve direct entry into protected. On a LIRS-style trace where that pattern is common, this signal is worth capturing.
- v003 was built from the baseline rather than from the cycle-1 SLRU winner, so its measured gain bundles both SLRU segmentation and ghost admission; the individual contribution of the ghost set is not yet isolated.

## Open questions

- How much of v003's gain is attributable to ghost admission alone versus SLRU segmentation? Applying ghost admission to the cycle-1 SLRU baseline (without rebuilding the segment logic) would isolate the effect.
- Could frequency-aware demotion recover if the scan window were much larger and the counter were persistent across demotion cycles rather than reset? The current failure is almost certainly a design flaw rather than a fundamental limit of the idea.
- Is ghost capacity equal to probationary capacity the right size, or would a smaller ghost window (e.g., half of probationary) suffice while reducing bookkeeping overhead?

## Continuation

Next cycle should take v003 as the new baseline and add ghost admission on top of the established cycle-1 SLRU structure as a control variant, to cleanly measure the ghost set's marginal contribution. It is also worth retrying frequency-aware demotion with a larger scan window and non-resetting counters before abandoning the idea; the cycle-2 failure was a signal-design problem, not proof that frequency is uninformative in the protected segment. Further ratio tuning of the 80/20 split is likely a diminishing return and can be deprioritized.
