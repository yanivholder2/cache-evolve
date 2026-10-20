## Recurring wins

- Landing stash-hit promotions at the MRU end of the target bucket is the one change that has moved hit-rate in the right direction (cycle 1), and it has been kept as the base for later stash work.
- Requiring a second hit before a stash item is promoted gave a small hit-rate gain on top of MRU landing (cycles 2 and 3). The evidence is one trace at one capacity.
- Behaviour-preserving refactors (occupancy counters, store-skip, gate reordering) have matched their parent exactly. That is a sound way to keep correctness, but it is not a performance win until cost is measured.

## Recurring failures

- Reuse-counter refinements have been flat for four cycles (cycles 6 to 8): saturating counters, back-to-back gating, halving on demotion, and saturated-gate promotion. None moved hit-rate beyond noise on LIRS workload-a at size 2048.
- Changes that move or re-insert proven-hot items hurt. Removing and re-inserting probing-tier hits (cycle 1) pushed hot keys back through target competition.
- Skipping the target retry for foreign evictions hurt hit-rate (cycle 1). Gating it on stash capacity recovered only part of the loss (cycle 2), so the retry matters under stash pressure.
- Using the seen-in-stash marker as an eviction preference hurt (cycle 3). It discarded recently arrived items that had not yet had a chance to prove reuse.
- Shrinking the probing tier to grow the stash lost hit-rate (cycle 8). Stash pressure does not look like the binding constraint on this trace.
- Early readmission via a ghost-history fast path gave a slight hit-rate loss (cycle 5).
- Landing-slot, victim-order, and occupancy-conditional refinements (cycles 2 and 4) showed no effect. Demoted-item placement (cycle 5) also showed no effect.

## Open questions

- Metadata-size and per-op-time have not been measured in any cycle. Every claimed cost benefit (occupancy counters, spare-bit marker, reuse-counter bits) is unverified. This is the largest gap.
- Do the MRU landing and second-hit gate hold on workloads and capacities other than LIRS workload-a at size 2048?
- Several "Winner" labels do not match their measured results (cycles 4, 5, 7, 8, and the inherited v001 in cycle 2). Is the label carried forward, or is it winning against a different baseline? Until this is clear, treat Winner status as weak evidence.
- Many insights describe intended changes, not verified implementations, because overlay diffs were not readable. Implementations remain unconfirmed.
- Would a larger probing tier, the opposite of the cycle 8 shift, help, or is the current split already reasonable?

## Per-policy notes

- Stash tier (promotion and landing): MRU landing plus a second-hit gate is the only lead with a hit-rate signal. Keep it, and test it more broadly.
- Stash tier (victim selection): marker-based victim preference hurt; earliest-landed selection matched baseline. Victim-order tweaks have not helped.
- Probing tier: remove-and-reinsert of hits hurt; zero-hit exit filtering and shrinking the tier did not help. Residency in probing appears valuable.
- Normal tier and demotion: LRU-end placement and reuse-counter reset or halving on demotion were flat. No signal here yet.
- Foreign eviction: skipping the target retry hurt; the retry matters under stash pressure. Occupancy-counter victim search matched parent, so it is a cost-only experiment.
- Ghost history: readmission fast path lost slightly. Revisit only with a second workload showing a clear gain.
