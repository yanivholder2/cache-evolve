## Recurring wins

- Changes that decide which keys get in or out beat changes that decide where keys sit. Two real steps occurred: a second-sighting admission filter (cycle 3) and reuse-based victim selection (cycle 6).
- A real reuse signal (hit since insertion) used for victim choice broke the plateau left by placement and admission work. Finer state on top of it (a 2-bit aged counter) added only a modest gain.
- Scan-time aging, tied to where the evictor looks, is a useful idleness signal.
- MRU placement on hit-promotion (cycle 1) was the only placement lever that ever paid off. It was subsumed afterwards.

## Recurring failures

- Placement and position tweaks (insert, demotion, eligibility, two-tier insert, power-of-two buckets, wider probing) were flat or slightly negative. They have been saturated since cycle 2.
- Refining the admission filter (counters, doorkeepers, bigger or smaller filters, victim exemptions, adaptive gates, count-min) never beat the plain one-bit filter. Admission mechanisms also overlap rather than stack.
- Extra history or tiers (ghost table, protected bit, wider counters) gave noise-level gains or small losses. Over-protecting stale entries hurts.
- Global-timer aging, scan-order changes and free-slot admission bypass were flat or negative.
- Cycles 4 and 5 and 7-10 produced mostly noise-level outcomes. Winner labels in those cycles are likely selection noise, not real improvements.
- Several edits reproduced the parent's hit-rate exactly. They were probably dead or off-path overlays, which makes the null results ambiguous.

## Open questions

- Does any hit-rate headroom remain on workload-a, or is everything at the noise floor?
- Do the admission and eviction-reuse gains hold on other workloads and cache sizes?
- What are the per-op-time and metadata-size costs? These have never been measured. Simplifying variants and compact designs cannot be judged without them.
- Are noise-level Winner labels a flaw in the selection rule, or a missing fitness dimension?
- Are there untested levers outside admission, placement and eviction state, such as adaptive sizing or scan-resistance for other access patterns?

## Per-policy notes

- Baseline LRU-like segment (promote on hit, evict from the cold end): placement variants barely moved it. Victim choice and admission were the real levers.
- Second-sighting admission filter (cycle 3): it is the incumbent filter. Keep it unconditional and one-bit. Further tuning has failed.
- Reuse-bit and counter eviction (cycles 6-7): this is the main gain over LRU. Further refinements (scan order, ghost table, tiers, wider counters, bucket drop rule) are at the noise floor.
- All evidence comes from one trace and one cache size, with hit-rate only. Mechanism and cost claims are inferred, because the authors could not read the overlay sources.
