## Recurring wins

- Second-hit admission (keep one-touch items out of hot tiers) is the one real policy win. It was confirmed independently in cycles 0001, 0002 and 0004, and it held at flat metadata once the flag was bit-packed.
- A small admission filter sized to one cache line and aged often enough to stay sparse (cycle 0008) is the best put-side result. It skips insert and eviction work on one-hit keys and holds hit-rate.
- Removing bookkeeping that the policy never reads helped once (cycle 0004). Later deletion-only variants gave nothing, so that gain is spent.
- Metadata stays flat in nearly every variant, because spare bits are enough. Footprint is not a constraint.

## Recurring failures

- Hot-path or control-flow edits to the incumbent (lookup atomics, early returns, free-slot fast paths, promotion skips, hoisting, tag-first scans, reader-writer locks, put early-return) regressed latency in about seven cycles in a row. They add branches or work, and the benchmark appears uncontended.
- Latency regressions often show up on paths the change did not touch. This points to layout effects or noise, and the noise floor has never been measured.
- Extra admission or recency signals (hit counters, ref bits, ghost fingerprints) gave negligible hit-rate, because the second-hit filter already screens most one-touch items.
- Insert-position tweaks overlap with admission and do not stack. Position alone looks exhausted.
- Lossy, unaged or tiny filters buy latency at the cost of hit-rate (cycles 0006 and 0008). Large filters recover hit-rate but cost put latency.
- Doorkeeper tuning (aging threshold, probe count) is now a weak lever.
- Winner status is weak evidence. Flat hit-rate and metadata are enough to qualify, even when latency regresses.
- Variant authors often could not read the source, so mechanisms are inferred from fitness vectors.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? It has been asked for six cycles and is still unanswered. Measure it with repeated runs, comparing against the parent in the same run.
- Where does get and put time actually go? No profiling has been done.
- Do the conclusions hold on a second trace with scans or frequency skew? All results so far are from the recency-skewed LIRS trace.
- Would eviction or promotion changes, rather than admission, move hit-rate further? Admission looks saturated.
- Are the small hit-rate gains of the aged doorkeeper variants real or within trace noise?

## Per-policy notes

- DASH segment, second-hit / doorkeeper admission: this is the active family. The current incumbent is the one-cache-line aged two-probe doorkeeper from cycle 0008. It is tuned out, so stop further tuning.
- Insert position (uniform MRU, cold/hot split, cold demotions): small gains at best, with a hit-rate and latency trade-off. Do not stack on admission.
- Eviction order (CLOCK-style second chance): no hit-rate gain and a latency cost.
- Locking and bucket scan: no sign of a bottleneck. Do not revisit without profiling.
- Next direction: measure noise and profile first, then try eviction or promotion policy on a second trace.
