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
