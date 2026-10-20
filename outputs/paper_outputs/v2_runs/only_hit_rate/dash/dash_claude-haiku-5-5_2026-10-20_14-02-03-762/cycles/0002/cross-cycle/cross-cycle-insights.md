# Cross-cycle insights

## Recurring wins

- Changes to where and whether stash items enter or land have produced the only hit-rate gains so far: MRU landing for re-referenced stash items (cycle 0001) and requiring a second hit before stash promotion (cycle 0002). Both act on stash admission or placement rather than on movement.
- Admission filtering is the stronger lever so far: gating promotion on a repeat hit is consistent with one-off stash hits displacing resident items.

## Recurring failures

- Adding movement or removing a retry keeps losing hit-rate: remove-and-reinsert on probing hits (0001), skipping the target retry for foreign evictions (0001), and an unconditional direct-to-stash shortcut (0002).
- Partial fixes to failed shortcuts recover little: gating the direct-to-stash path on free stash capacity recovered only a small part of the lost hit-rate (0002), and an occupancy-conditional landing showed no measurable effect (0002).
- Every cycle judged changes on hit-rate alone. Per-op-time and metadata-size have never been measured, so no cost of any change is known.
- Several insights rest on hypothesis text rather than the implemented diff, so reported behaviour is not confirmed.

## Open questions

- Do the stash-admission and MRU-landing gains hold on workloads other than the single LIRS trace used so far?
- Does the second-hit promotion filter beat the baseline, or only its parent? The 0002 winner's hit-rate matched its parent, so its gain over baseline is inherited, not newly shown.
- What do per-op-time and metadata-size cost under the promotion filter and its seen-in-stash marker?
- Does the removed target retry pay off once stash capacity handling is improved? Stash pressure looks relevant, but this is untested.
- Is the occupancy-conditional landing ever triggered on any workload?

## Per-policy notes

- The ledger does not break results down by eviction policy. All evidence comes from the LIRS trace within the probing / stash / normal tier scheme, so policy-specific regularities cannot yet be separated from workload effects.
- Within this tier scheme, admission and landing changes to the stash have been the productive area; movement and retry-removal changes have not.
