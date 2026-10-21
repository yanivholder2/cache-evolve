## Recurring wins

- Levers that use a real reuse or value signal pay off. Admission (second-sighting filter, cycle 3) and reuse-based victim choice (reference bit, cycle 6) were the only large movements.
- Deciding what to admit or evict beats deciding where to place. Placement changes only won once (MRU on hit, cycle 1).
- Gains shrink as state gets richer. A counter with aging on top of the reuse bit gave only a modest gain (cycle 7).
- Structurally different levers have broken plateaus. Refinements of the same lever have not.

## Recurring failures

- Placement and position tweaks (insert, demotion, eligibility, two-tier insert, power-of-two buckets, wider probing) are flat or negative. Placement is saturated on workload-a.
- Refining the admission filter (counters, doorkeeper, victim exemption, larger capacity, count-min, bypass when free slots exist) is flat or negative. The one-bit unconditional filter is a local optimum. Stacking admission mechanisms overlaps and does not add up.
- Scan-order and second-chance tweaks on eviction have no effect.
- "Winner" labels are often noise-level or only match the parent (cycles 2, 4, 5). They are not reliable evidence of improvement.
- Several variants reproduced the parent exactly. The overlay edits may not have been on the live path, and authors could not read the sources, so mechanism claims are inferred.

## Open questions

- Do the gains hold on other workloads and cache sizes? All evidence comes from one trace at one size.
- What do the wins cost in per-op-time and metadata? Only hit-rate has ever been measured, and the eviction scan and aging may be costly.
- Is any hit-rate headroom left on this trace? Adaptive sizing, scan-resistance and other levers outside admission, placement and eviction state are untried.
- Is the Winner rule relative to the cohort? Is the fitness vector missing dimensions?
- Does victim selection that is confirmed live behave differently from the earlier no-op results?

## Per-policy notes

- Baseline with MRU-on-hit: a small early win. Further placement variants on top of it add nothing.
- Second-sighting admission filter (cycle 3 v002): a solid gain. Treat it as fixed and keep it unconditional.
- Reuse-bit eviction (cycle 6 v001): the largest gain so far. It needs no extra metadata in principle, since the flag lives in spare bits.
- Two-bit aged counter eviction (cycle 7 v001): the current incumbent, a small win over the reuse bit. Its scan cost is unmeasured.
- Count-min, TinyLFU-style and doorkeeper admission: little or no gain over the one-bit filter.
- Power-of-two-choices and wider-probe structures: flat or slightly negative.
