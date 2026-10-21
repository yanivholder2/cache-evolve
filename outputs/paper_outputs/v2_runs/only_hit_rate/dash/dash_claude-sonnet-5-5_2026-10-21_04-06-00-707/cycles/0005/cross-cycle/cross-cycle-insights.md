## Recurring wins

- Two wins in five cycles, both on workload-a: MRU placement on hit-promotion (cycle 1) and the second-sighting admission filter (cycle 3). Each came from a lever that changed what is kept or where hit entries land, without adding item movement.
- Screening out one-hit-wonders at admission is the strongest lever found so far. Placement tweaks never matched it.

## Recurring failures

- Placement-position tweaks (insert, demotion, eligibility, two-tier insert) were flat after cycle 1. Every variant matched its parent's hit-rate. Either the levers are saturated or the overlays were dead edits.
- Extra item movement (bidirectional swaps, more promotions) and delayed or relocated eviction (wider probe) gained nothing or lost a little, because of demotion cascades and cold entries crowding out hot ones.
- Refinements of the admission filter (counters, victim-conditional exemption, doorkeeper, 4x capacity, stacking on another admission parent) all landed within noise or below the parent. Smaller or faster-resetting state lost accuracy. Overlapping admission mechanisms do not stack.
- "Winner" labels from cycle 2 onward were mostly noise-level or relative to a weak cohort, so they are unreliable as evidence of improvement.

## Open questions

- Do the cycle 1 and cycle 3 gains hold on other workloads and cache sizes? Everything so far is workload-a.
- What do the wins cost in metadata-size and per-op-time? Only hit-rate has ever been measured, so noise-level winners cannot be rejected on cost.
- Are the overlay edits on the executed path? Repeated exact ties with the parent suggest some are not. Victim selection in particular is untested on a confirmed live path.
- Is there an untried structural lever, such as bucket and probing structure or a different admission signal, that moves hit-rate more than noise?
- Is the Winner rule relative to the cohort, or is the fitness vector missing dimensions?

## Per-policy notes

- Promotion (hit path): MRU insertion on hit helps. Eligibility relaxation and back-promotion add little or nothing.
- Admission: the one-bit unconditional second-sighting filter (cycle 3 v002) is the incumbent, and tuning it is exhausted on workload-a.
- Eviction and victim selection: a furthest-from-MRU scan tied its parent exactly, which is inconclusive. Cold-end demotion also showed no effect.
- Probing and stash: a wider neighbourhood before the stash slightly hurt. Bucket and probing structure is otherwise unexplored.
