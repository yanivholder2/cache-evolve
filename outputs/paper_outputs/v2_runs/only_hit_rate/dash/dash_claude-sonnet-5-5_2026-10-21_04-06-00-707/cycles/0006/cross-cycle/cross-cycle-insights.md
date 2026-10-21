## Recurring wins

- Levers that change *which* entries stay resident beat levers that change *where* they sit. The two clear gains were the second-sighting admission filter (cycle 3) and reuse-bit victim selection (cycle 6).
- Promotion position (MRU on hit) was the only placement change that helped (cycle 1). It did not stack with later placement tweaks.
- Using a real reuse signal (hit since insertion) for eviction broke the plateau that admission and placement tuning had reached.
- Simple, near-zero-metadata mechanisms have outperformed more elaborate ones.

## Recurring failures

- Placement-position tweaks (insert, demotion, eligibility, two-tier insert) have been flat or noise-level since cycle 2. They are saturated on workload-a.
- Refining or stacking admission machinery (counters, doorkeepers, victim exemption, larger filters, count-min gates) lost or gained only noise. The one-bit filter looks like a local optimum, and overlapping admission mechanisms do not add up.
- Extra movement (bidirectional swaps, wider probe neighbourhoods, power-of-two bucket choice) gave no gain and sometimes cost hit-rate through cascades or by keeping cold entries resident.
- Several results matched the parent's hit-rate exactly, which suggests dead or non-live overlay edits (mostly victim selection in cycles 2-3).
- Winner labels were sometimes assigned to noise-level or sub-parent variants (cycles 4, 5), so the winner rule is unreliable without cost dimensions.

## Open questions

- Do the admission and reuse-bit eviction gains generalize to other workloads and cache sizes? Every result so far comes from workload-a.
- Can a richer reuse signal do better than one bit (multi-bit aging, CLOCK-style second chance, scan resistance)? Does it stack with the admission filter?
- What are the per-op-time and metadata costs of the eviction scan, the filter state and the extra probes? None have been measured, because only hit-rate has been reported.
- Are the overlay edits reaching the executed path? Authors cannot read the sources, so mechanism claims remain inferred.
- Is the winner-selection rule relative to the cohort, or is the fitness vector missing dimensions?

## Per-policy notes

- Baseline/promotion (cycle 1): MRU insertion on hit helped. Added promotion movement did not.
- Admission (second-sighting one-bit filter, cycle 3): the incumbent for admission. Variants (counter, doorkeeper, victim-conditional, 4x capacity, count-min) were all flat or worse.
- Eviction (reuse-bit, first unreferenced slot, cycle 6): the current best and incumbent. Furthest-from-MRU scan (cycle 3) and cold-end demotion (cycle 2) showed no effect, possibly because they were not live.
- Bucket/probing structure: wider probe and power-of-two choices gave no benefit. Treat the structure as low-yield.
- Next direction: branch from the reuse-bit eviction incumbent, keep the admission filter, and add cost metrics to the fitness vector.
