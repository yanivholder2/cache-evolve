## Recurring wins

- Changes that decide *what* is kept beat changes that decide *where* it goes. The big moves were a second-sighting admission filter (cycle 3) and reuse-signal victim selection (cycle 6). The only early placement win was MRU on hit-promotion (cycle 1).
- Using a real reuse signal (hit since insertion) for eviction broke a multi-cycle plateau. Richer state on top of it (aged counter) added only a small gain.
- Keeping aging local to the evictor's scan beat global timer-driven aging.
- Keeping the admission filter unconditional held up. It is not redundant with reuse-bit eviction.

## Recurring failures

- Placement and position tweaks (insert, demotion, eligibility, two-tier insert, power-of-two buckets, wider probing) were flat or negative. They are saturated on workload-a.
- Refining or stacking admission machinery (counters, doorkeepers, bigger filters, count-min, victim exemptions, free-slot bypass) was flat or negative. Overlapping admission mechanisms do not add up.
- Extra history or tiers (ghost table, protected bit, ghost filtering) gave noise-level gains at best, with unmeasured cost.
- Scan order and second-chance fallback had no effect.
- Since cycle 4, most Winner labels are noise-level or relative to the cohort only. They are weak evidence of improvement.
- Several variants matched their parent exactly. Some overlay edits may have been dead code, and authors could not read the sources.

## Open questions

- Does any gain, especially reuse-based eviction and admission, hold on other workloads and cache sizes? All evidence is one trace and one size.
- Is there any hit-rate headroom left on workload-a, or is every lever at the noise floor?
- What are the metadata-size and per-op-time costs of the filters, counters, ghost state and scan work? Per-op-time and metadata-size have never been in the fitness vector, so small wins cannot be judged on cost.
- Would a compact design (one-bit flag with bit-array filter) win on cost despite a marginal hit-rate gain?
- Are there untried levers outside admission, placement and eviction state, such as adaptive sizing or scan-resistance for other access patterns?

## Per-policy notes

- Baseline LRU-like promotion: MRU on hit-promotion helped. More promotion movement (eligibility, bidirectional swaps) did not.
- Admission (second-sighting one-bit filter): this was the first real win, and it is the incumbent design. Capacity, aging, counters, victim conditioning and frequency sketches are all at or below noise.
- Eviction (reuse bit, then aged 2-bit counter): this gave the largest and most recent real gain, and refinements since are diminishing. Ghost restore is marginal. Global-timer halving hurt. Scan order is irrelevant.
- Bucket and probing structure: wider probing and power-of-two choices were no help. Dropping an incoming key when its bucket is all hot did almost nothing.
- Compact one-bit reuse flag with bit-array filter: the hit-rate gain is marginal, and its cost savings are untested.
