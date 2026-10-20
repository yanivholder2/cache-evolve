# Cross-cycle insights

## Recurring wins

- Gating stash promotion on re-reference is the one lever that has repeatedly shown a hit-rate signal. Requiring a second hit (cycle 2) and stacking that gate on MRU landing (cycle 3) both helped, though only on the single trace.
- Landing a re-referenced stash item at the MRU end of its target bucket is the original hit-rate gain (cycle 1). It has been carried forward unchanged since, and no later cycle has improved on it.
- Probing-tier residency is the binding resource. Shrinking the probing tier to grow the stash lost hit-rate (cycle 8), and reversing that at fixed total capacity gained it (cycle 9). The two results agree in direction.

## Recurring failures

- Reuse-counter refinements have produced no robust signal for four cycles (6, 7, 8, 9). Increment rules, reset-on-demotion, saturation gates, and collapsing the counter back to a one-bit marker all landed at parent level or within noise.
- Using the seen-in-stash signal as an eviction preference hurt (cycle 3). It discarded recently arrived items that had not yet had a chance to prove reuse.
- Early readmission of recently evicted keys, via a ghost-history path that skips the wait, cost a little hit-rate (cycle 5).
- Movement-adding or retry-removing changes regressed. Remove-and-reinsert on probing hits, direct-to-stash eviction, and skipping the target retry for foreign evictions all lost hit-rate (cycle 1). A capacity-gated variant of the shortcut recovered only part of that loss (cycle 2).
- Stash-victim ordering and occupancy-conditional landing or gating tweaks have been flat (cycles 2, 4). Eviction-order and landing refinements in this part of the design have shown no effect so far.
- Behaviour-preserving refactors (occupancy counters, skipped slot scans, store-skip, gate reorder) match their parent by construction. Their claimed speed benefit has never been measured, so they have not shown value.

## Open questions

- Do the winning levers (re-reference promotion gate, MRU stash landing) hold on workloads and capacities other than LIRS workload-a at size 2048? No cycle has tested a second trace.
- Where is the probing-tier optimum? The gain from growing probing may flatten or reverse beyond 128 slots.
- Does per-op-time or metadata-size move under any change? Neither has been recorded in any of the nine cycles, so the cost side of every variant is unknown, including the bit cost of any reuse counter or marker.
- Several cycles are labelled Winner with hit-rate at or below the parent (cycles 4, 5, 7, 8). Which baseline these wins are measured against is unclear, so the winner labels should not be read as evidence of improvement.
- Most insights describe intended changes rather than verified overlay source. Whether the implementations match their descriptions is unconfirmed.

## Per-policy notes

- All nine cycles evaluate one policy family: a tiered design with probing, stash, and normal tiers, judged by hit-rate on LIRS workload-a at size 2048. The ledger has no per-policy split, so the notes below apply to that family only.
- Admission and promotion (probing → stash → normal): re-reference gating is the most promising direction. Probing-tier admission filters and back-to-back filters were flat.
- Landing and placement: MRU landing for stash hits is established. Demoted-item placement at the LRU end was flat (cycle 5).
- Eviction and victim selection: stash victim preferences (never-re-hit, earliest-landed) were harmful or flat. Foreign-eviction retry removal regressed.
- Reuse tracking (counter or marker): no robust signal across cycles 6–9. Treat as low priority unless a second workload shows a gain.
