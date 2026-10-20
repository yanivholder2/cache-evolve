## Recurring wins

- Landing position in the target bucket is the lever that has helped. MRU landing for stash hits was the only hit-rate gain in cycle 1, and stacking the second-hit gate on it gave a small gain in cycle 3.
- Admission filtering helps. Requiring a second hit before stash promotion improved hit-rate in cycle 2, suggesting one-off stash hits were displacing resident items.

## Recurring failures

- Adding movement or removing a retry tends to hurt. Re-inserting probing hits (cycle 1) and the unconditional direct-to-stash eviction shortcut (cycles 1 and 2) both lost hit-rate. Gating the shortcut on free stash capacity recovered only part of the loss, so the removed retry matters under stash pressure.
- Using the seen-in-stash marker as an eviction signal backfires. Preferring never-re-hit stash entries as victims discarded recently arrived items that had not yet had a chance to prove reuse (cycle 3).
- Fine-grained refinements showed no measurable effect. Occupancy-conditional landing (cycle 2), occupancy-conditional promotion gating (cycle 4), earliest-landed victim selection (cycle 4), and moving the marker into a spare tag bit (cycles 3 and 4) all matched their parent exactly on hit-rate.

## Open questions

- Do the MRU landing and second-hit gate hold on workloads and capacities other than the single LIRS trace at size 2048?
- Per-op-time and metadata-size have not been measured in any cycle, so the cost side of the marker, promotion gate, and shorter eviction path is unknown.
- Did the spare-bit variants actually change the entry layout? Several insights rest on hypothesis text rather than the overlay diff.
- Some "Winner" labels (cycles 2 and 4) carry no change from the parent. Is the win against the baseline or only inherited?
- Would the removed target retry pay off if paired with better stash capacity handling?

## Per-policy notes

- The ledger covers one tiered design (probing, stash, normal tiers) evaluated on a single LIRS trace at size 2048. No other eviction policy appears, so no cross-policy comparison is possible yet.
- Within the stash tier, promotion and landing decisions (MRU landing, second-hit gate) are the only ones with a positive signal.
- Within the stash tier, victim-selection changes (never-re-hit preference, earliest-landed) have not improved hit-rate.
- Probing-tier residency should be preserved; removing and re-inserting probing hits lost ground.
