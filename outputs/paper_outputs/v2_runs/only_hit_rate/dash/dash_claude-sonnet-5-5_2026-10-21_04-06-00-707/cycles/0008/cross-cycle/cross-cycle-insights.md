## Recurring wins

- Changing *what gets kept or dropped* beat changing *where things sit*. The two clear movers were the second-sighting admission filter (cycle 3) and reuse-signal victim selection (cycle 6).
- Using a real reuse signal (hit since insertion) for eviction gave the largest gain since admission. Finer state on top of it (counters, aging, ghost history, tiers) added only small increments.
- Local, scan-time aging tied to where the evictor looks works. Global timer-driven aging did not.
- Keeping the admission filter unconditional has held up. It is not redundant with reuse-based eviction.

## Recurring failures

- Placement and position tweaks (insert, demotion, eligibility, two-tier insert, power-of-two buckets, wider probes) were flat or slightly negative from cycle 2 on. Once admission screens one-hit-wonders, ranking within a bucket adds nothing.
- Refining or stacking admission machinery (counters, doorkeepers, victim exemption, bigger filters, count-min gates, free-slot bypass) lost or tied. The incumbent filter looks like a local optimum, and overlapping admission mechanisms are not independent wins.
- Scan-order and second-chance tweaks were identical to the parent. History and ghost state gave near-noise gains.
- Several variants matched their parent's hit-rate exactly, so some overlay edits may have been dead code paths. This weakens negative conclusions about those levers.
- Winner labels often go to noise-level or even below-parent variants (cycles 4, 5, 8). Selection is relative to the cohort and is not a real-gain signal.

## Open questions

- Do the admission and eviction gains hold on other workloads and cache sizes? All evidence is from workload-a at one size.
- Is any hit-rate headroom left on workload-a, or is it saturated across admission, placement and eviction state?
- What do the winners cost in per-op-time and metadata (filter memory, counter aging on scan, ghost table, multi-pass scans)? None of this has been measured. Hit-rate is the only fitness dimension so far.
- Are there untried levers outside admission, placement and eviction state, such as adaptive sizing or scan-resistance for other access patterns?
- Are overlay edits reliably on the live path? The authors cannot read the sources.

## Per-policy notes

- Baseline promotion/insertion: MRU placement on hit helped once (cycle 1). Every further placement variant was neutral.
- Admission (second-sighting one-bit filter): the cycle 3 incumbent. Capacity, aging, counters, doorkeepers and frequency gates did not beat it. It does not stack on parents that already screen inserts.
- Eviction (reuse-bit victim choice, cycle 6): a large step that is now the base. A two-bit aged counter gave a modest gain (cycle 7). Ghost restoration gave a marginal gain (cycle 8). Tiers gave a modest gain on the one-bit parent. Scan order and global halving gave none or a loss.
- Current incumbent: the counter-based reuse eviction with the unconditional admission filter. It is a marginal-gain line with unmeasured cost.
