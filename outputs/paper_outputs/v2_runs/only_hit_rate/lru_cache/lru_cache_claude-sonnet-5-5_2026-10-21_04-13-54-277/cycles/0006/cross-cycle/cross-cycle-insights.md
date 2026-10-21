## Recurring wins

- Segmented LRU (probation + protected) is the one design line that has consistently produced winners; scan resistance comes from keeping one-hit keys in a small probation segment.
- Raising the protected share paid off in successive cycles, with diminishing returns, then a plateau.
- Counting rejected misses in an admission sketch helps: recurring keys build frequency across retries and earn admission.
- CLOCK variants (one-bit second chance) give small hit-rate gains over strict LRU without list manipulation.

## Recurring failures

- Weakening the protected segment (smaller share, ghost-to-protected re-admission) loses hit-rate.
- Probation-side tweaks (demotion placement, frequency-aware eviction scans) are inert at a high protected share: demotions are rare, and frequency state was not persisted, so scans fell back to LRU order.
- Protected-eviction refinements (second-chance bit) and approximate-recency shortcuts (relink-skip) are inert or slightly negative.
- Admission-gate tuning (sketch update rule, doorkeeper, thresholds) has plateaued; segmentation already filters most scans.
- Frequency-aware CLOCK and CLOCK + sketch are only incremental; they have not beaten the segmented line.
- Many variants matched their parent exactly, meaning the changed code path rarely ran on the evaluated trace.
- Metadata-size and per-op-time have been absent from every fitness vector in every cycle. Selection has been on hit-rate alone, with marginal differences, and all cost claims are unverified.

## Open questions

- Is the missing cost measurement (metadata-size, per-op-time) a harness fault? It blocks judging every variant and should be fixed first.
- Do the gains hold on other traces and cache sizes, especially those with long-distance reuse or little scan pressure? All evidence comes from one scan-heavy trace at one size.
- Would adaptive (ARC-style) segment sizing beat any fixed share?
- Would frequency persisting across segments or covering protected entries help?
- How does CLOCK + sketch compare head to head with the segmented winner, on hit-rate and cost?
- Are the sketch gate and doorkeeper worth their added state and time?

## Per-policy notes

- Segmented LRU: saturated on this trace at about a 95% protected share. Stop tuning the share, probation placement, and protected eviction.
- Admission gate (TinyLFU-style sketch, doorkeeper): marginal gain, plateaued; the cost is unmeasured. Prefer cheaper variants only if cost can be measured.
- CLOCK (one-bit, frequency, + sketch): small gains. Its size and speed advantages are the main claim but are unverified.
- Strict LRU: micro-optimizations were neutral on hit-rate and cannot win without cost metrics.
- Ghost list: harmful when it admits directly to protected; neutral when it re-admits at probation MRU. Revisit only with a long-reuse trace.
