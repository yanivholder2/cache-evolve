# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0010

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_14-11-42-199/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cohort tried victim-end placement for admitted keys (v001), pruning singleton history from the shared frequency map (v002, the recorded Winner), and quadrupling the second-miss ghost-list window (v003). All three report a hit-rate regression on workload-a, so no variant produced a gain, and the Winner's own insight describes a decrease.

## Patterns

- No variant improved the measured dimension. The Winner label (v002) again conflicts with its evidence, as in Cycles 0004, 0006, and 0007. The selection rule should be checked before v002 is used as a precedent.
- Victim-end placement for admitted keys (v001) lost hit-rate again, now in a second cycle after Cycle 0007's version. The Cycle 0005 eviction-next gain has not reproduced under this placement. Its insight suggests the earlier gain depended on back-of-probation placement, so the two results may not be comparable.
- Widening the ghost-list window (v003) regressed. Together with Cycle 0007's recurrence gate, this suggests the one-window setting is about right and a longer first-miss memory admits stale one-shot keys.
- Pruning singleton frequency history (v002) produced a small regression, not the metadata saving its hypothesis predicted. The change also has no hysteresis, so a full prune could run on many accesses. That is a plausible per-op cost, but it is not measured.
- Metadata size and per-op time are still unmeasured in every variant, as in Cycles 0001 to 0007. The fitness vector still lacks these dimensions.

## Open questions

- Why was v002 chosen as Winner when its insight reports a hit-rate decrease?
- Does the Cycle 0007 recurrence-gate gain (ghost list at one-capacity window) hold when the window is kept at one capacity but the rest of the setup is unchanged, or was that gain specific to its run?
- Does singleton pruning change the admission gate's behavior enough to explain the decrease, or is the loss from some other effect of the prune?
- How often does the prune's full pass run under the current map size, and what does that cost per access?

## Continuation

Next cycle should not pursue victim-end placement, ghost-window widening, or singleton pruning as hit-rate levers. Keep the one-capacity ghost window from Cycle 0007 as the base for recurrence gating, and retest its gain before building on it. Reconfirm the Winner selection before using v002 as a precedent, and add metadata size and per-op time to the fitness vector before accepting any memory-saving change.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

The cohort tried a first-miss ghost refusal placed in front of the shared-frequency admission gate (v001, the recorded Winner), eviction-next placement for keys that pass the gate (v002), and periodic halving of shared frequency counters with zero-entry pruning (v003). All three report a hit-rate regression on workload-a, so no variant produced a gain, and the Winner's own insight describes a loss.

## Patterns

- No variant improved the measured dimension. The Winner label (v001) conflicts with its evidence, as in Cycles 0004, 0006, 0007, and 0008. The selection rule needs checking before v001 is used as a precedent.
- Stacking a recurrence gate in front of the counter gate (v001) lost hit-rate. The counter comparison already bypasses one-shot keys, so the ghost list mostly delays admission for keys that would have been admitted. This does not contradict Cycle 0007's ghost-list gain, but it shows that the recurrence signal did not add information on top of the counter gate.
- Eviction-next placement for admitted keys (v002) regressed again, now paired with the gate. Victim-end placement lost in Cycles 0007 and 0008 as well, so placement changes that make newly admitted keys eviction-eligible sooner have not helped on this workload.
- Halving counters with zero-pruning (v003) removed history for every key seen only once per window. This is the same singleton-history loss that Cycle 0008 found from pruning, so the loss now shows up under a second mechanism and is more likely a property of the removal than of the prune's trigger.
- Metadata size and per-op time are still unmeasured in every variant, as in Cycles 0001 to 0008. The cost side of the fitness vector remains empty.

## Open questions

- Why was v001 chosen as Winner when its insight reports a hit-rate loss?
- Would a recurrence signal that replaces the counter rejection, rather than preceding it, behave differently from the stacked form tested here?
- Does halving without removing entries preserve the one-hit information that the zero-pruning step discards?
- Was the v002 mechanism (eviction-next makes admitted keys the first victim candidate) confirmed against the diff? Its insight says the overlay source was not readable, so the mechanism is inferred from the hypothesis.

## Continuation

Next cycle should not stack a first-miss refusal in front of the shared-frequency gate, and should not pursue eviction-next or victim-end placement for admitted keys. Drop zero-pruning in any aging scheme; if aging is retested, halve without removing entries and test it separately from the admission rule. Reconfirm the Winner selection before using v001 as a precedent, and add metadata size and per-op time to the fitness vector.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
