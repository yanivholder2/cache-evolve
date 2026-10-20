## Recurring wins

- Stash-hit admission and landing rules are the most productive lever. An MRU landing for re-referenced stash items (cycle 0001) and a second-hit gate before stash promotion (cycles 0002–0003) each produced a hit-rate gain, and they stacked modestly.
- Probing-tier capacity near 128 slots is the best-supported size. Shrinking probing hurt (0008), growing it by 128 slots helped (0009), and growing further to 256 hurt (0010).
- Keeping proven-hot keys resident in the probing tier matters. Removing and re-inserting probing hits lost ground (0001).

## Recurring failures

- Reuse-counter refinements show no robust signal. Increment rules, reset-on-demotion, back-to-back filters, store-skip, and saturated-counter gates were flat or within noise across cycles 0006–0008, and removing counter states gave only a small drop (0009).
- Recency-based or victim-preference gates hurt. A close-in-time second-hit promotion gate lost hit-rate (0010), and preferring never-re-hit stash entries as victims discarded items that had not yet had a chance to prove reuse (0003).
- Shortcutting or removing the demotion/eviction path hurt. Skipping the target retry for foreign evictions and sending probing exits straight to the stash both lost hit-rate (0001, 0002 for the unconditional form).
- Early readmission of recently evicted keys (ghost-history fast path) gave a slight loss (0005).
- Pure refactors and placement tweaks match the parent and do not move hit-rate. This covers occupancy counters, gate reordering, marker relocation and folding, earliest-landed victim order, occupancy-conditional landing and gating, and demotion-to-LRU placement (0002, 0004, 0005, 0006, 0010). Their claimed speed or memory benefits were never measured.

## Open questions

- Where is the probing-tier optimum? Is 128 slots the peak, or is there one between 128 and 256?
- Does the MRU landing and the second-hit gate hold on any workload or capacity other than the LIRS trace at size 2048? No cycle has tested a second workload.
- What are the per-op-time and metadata-size effects? Neither metric was recorded in any cycle, so every cost trade-off (including the bit cost of the reuse counter and the marker) is unknown.
- Several Winner designations do not match their measured results: 0005 v001, 0007 v002, 0008 v002, 0010 v003 were listed as Winners with flat or lower hit-rate, and 0002 v001 and 0004 v001 were carried forward without new gain. The baseline used for Winner status should be confirmed before building on these lineages.
- Several insights describe intended changes, not verified overlay diffs (0002, 0004, 0005, 0006, 0007, 0009). Implementation details, such as marker layout and reset timing, are unconfirmed.

## Per-policy notes

- All cycles evaluate one tiered design (probing, stash, normal) on the LIRS trace at size 2048. Per-policy comparison is not possible from this ledger; no other eviction policy was evaluated.
- Within this design, the robust signals are about tier sizing and promotion admission. Reuse-counter and eviction-order refinements have not shown a measurable effect.
