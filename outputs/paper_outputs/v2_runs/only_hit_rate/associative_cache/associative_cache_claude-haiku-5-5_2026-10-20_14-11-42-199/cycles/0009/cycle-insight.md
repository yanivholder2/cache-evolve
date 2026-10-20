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
