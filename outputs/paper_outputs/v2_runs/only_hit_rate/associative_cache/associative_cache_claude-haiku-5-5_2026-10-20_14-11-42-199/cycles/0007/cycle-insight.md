# Cycle 0007 Insight

## Cohort outcome

The cohort tried inserting admitted keys at the probationary victim end (v001), a second-miss ghost list that admits a key only on its second miss (v002), and a non-strict shared-frequency admission predicate that admits on counter ties (v003, the recorded Winner). Only v002 reported a hit-rate gain on workload-a; v001 and v003 both lowered it.

## Patterns

- The one positive result is a recurrence gate. The ghost list (v002) refuses first-time inserts without a frequency counter, which is the exact-key, no-counter form that Cycle 0006 asked to be retried. It also extends the one-shot-insert direction from Cycles 0005 and 0006.
- The recorded Winner (v003) has a hit-rate regression, and the only gain (v002) is not the Winner. This is the same Winner/evidence mismatch flagged in Cycles 0004 and 0006, and it now recurs. The selection rule needs checking before any of these results is used as a precedent.
- Insert position is a live confound. Placing admitted keys at the victim end (v001) lost hit-rate, while Cycle 0005's eviction-next insert reported a gain. The likely reading is that a newly admitted key needs some residency before it becomes eviction-eligible, but the Cycle 0005 and 0007 placements have not been shown to mean the same thing.
- Tightening the frequency predicate to admit ties (v003) gave no benefit. Strict comparison looks like the better default under unbounded counters.
- Cost is still unmeasured everywhere. v002's ghost list adds per-key metadata and v003 and v001 touch the shared counter, yet none of the three reports metadata size or per-op time. The hypothesis-level cost claims remain unverified, as in Cycles 0001 to 0006.

## Open questions

- Why was v003 chosen as Winner when its insight reports a slight hit-rate regression?
- Does the v002 ghost-list gain hold on a second workload, and what is its metadata size at the same cache size?
- Is the Cycle 0005 eviction-next gain the same mechanism as v001's victim-end loss, or is the placement semantics different?
- Does the v003 tie result reflect the tie rule itself, or a shift in victim selection pressure? The admitted-on-tie count was not checked.

## Continuation

Next cycle should pursue recurrence-gated admission (v002), testing its ghost-list window and checking it on a second workload with metadata size and per-op time in the fitness vector. Drop victim-end placement for newly admitted keys as a hit-rate lever, and do not pursue tie-admission in the shared-frequency predicate. Reconfirm the Winner selection before using v003 as a precedent.
