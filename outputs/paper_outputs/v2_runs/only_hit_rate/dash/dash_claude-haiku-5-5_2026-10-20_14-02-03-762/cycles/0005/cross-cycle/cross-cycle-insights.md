## Recurring wins

- Filtering which stash hits get promoted is the most consistent hit-rate lever. Requiring a second hit before promotion (cycle 2) and stacking it on MRU landing (cycle 3) both gave gains, though only on one trace.
- MRU landing for re-referenced stash items is the one landing change that helped (cycle 1). Refining it by occupancy (cycle 2) added nothing.
- Keeping proven-hot keys resident in the probing tier matters. Changes that remove and re-insert them (cycle 1) lost ground.

## Recurring failures

- Changes that add movement or remove a retry on the demotion path lower hit-rate. Remove-and-reinsert of probing hits (cycle 1) and unconditional direct-to-stash eviction without the target retry (cycle 1) both regressed. Gating the shortcut on free stash capacity (cycle 2) recovered only part of the loss.
- Using the seen-in-stash marker as an eviction signal hurts. Preferring never-re-hit stash entries as victims discards recently arrived items that have not had time to prove reuse (cycle 3).
- Admitting recently evicted keys early hurts slightly. The ghost-history fast path (cycle 5) let returning keys skip the seen-in-stash wait and lost hit-rate.
- Small tweaks to landing slot, promotion gate, victim order, and demotion placement have shown no hit-rate effect. These include occupancy-conditional landing (cycle 2), occupancy-conditional gating (cycle 4), earliest-landed victim selection (cycle 4), and LRU-end demotion placement (cycle 5). Stop spending cycles on this class of tweak.
- Layout or bookkeeping changes that leave hit-rate unchanged (spare-bit marker in cycle 3 and 4, occupancy counter in cycle 5) have not shown any realized saving. Their cost benefit is unmeasured.

## Open questions

- Winner status is inconsistent. Cycle 2's v001 matches its parent, cycle 4's v001 shows no change, and cycle 5's v001 shows a slight hit-rate drop. It is unclear what baseline each Winner is measured against.
- Per-op-time and metadata-size have never been measured in any cycle. The cost side of every change, including the marker, the ghost set, and the occupancy counter, is unknown.
- Does the MRU landing plus second-hit gate hold on workloads and capacities other than the LIRS trace at size 2048? All wins so far rest on that single trace.
- Several insights rest on hypothesis text rather than the overlay diff (cycles 2, 4, 5). Implementations are not confirmed as described.
- Does the foreign-eviction target retry pay for itself if stash capacity is handled better? Cycle 2 suggests stash pressure is what made it valuable.
- Rationale for the Front/Back default is described inconsistently across cycle 1 insights, which makes v002's intent hard to verify.

## Per-policy notes

The ledger does not name separate eviction policies. The notes below group observations by mechanism, and the only workload named is the LIRS trace.

- Probing tier: keep hits resident. Removing and re-inserting probing hits hurts. Demotion placement from this tier has shown no effect.
- Stash tier: promotion filtering and MRU landing are the only levers that moved hit-rate. Victim-selection changes (never-re-hit preference, earliest-landed) either hurt or had no effect. Capacity-gated shortcuts are a secondary lever whose cost is unmeasured.
- Normal / target-bucket tier: the target retry on foreign eviction has some value under stash pressure. Removing it unconditionally hurts. No measured variant has changed target competition in a way that helped.
- Ghost history: early readmission hurts on the one trace measured. Its memory cost is unknown.
