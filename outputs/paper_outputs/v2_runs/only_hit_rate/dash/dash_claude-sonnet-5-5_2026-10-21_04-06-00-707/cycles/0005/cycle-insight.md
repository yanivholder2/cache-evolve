# Cycle 0005 Insight

## Cohort outcome

Three variants were tried: a 4x larger, never-reset second-sighting filter on the cycle 3 incumbent (v001), a two-tier insert position that puts twice-seen keys at MRU and others next to MRU (v002), and a port of the one-bit filter onto the cycle 4 v002 variant (v003). v001 is the Winner, but only by a noise-level hit-rate gain. v002 was flat. v003 came out marginally below its parent.

## Patterns

- Every change landed within noise of its parent. Filter capacity, filter aging and insert position are all saturated on workload-a.
- Once the second-sighting filter screens out one-hit-wonders, the keys that get through are mostly worth keeping. Ranking them within a bucket adds nothing, which matches the cycle 2 finding that placement does not matter.
- The filter's gain did not stack on a different parent (v003). That parent already blocks some low-value inserts, so the two admission mechanisms overlap and are not independent wins.
- v001 bought a negligible gain with 4x the filter memory, so it is probably a bad trade. Memory is still unmeasured.
- Only hit-rate was reported again, and the authors could not read overlay sources. Cost claims and mechanism claims are still inferred.

## Open questions

- Is the Winner label for v001 meaningful, or is it noise-level selection again, as in cycle 4?
- Does the admission family hold up on other workloads or cache sizes, where filter capacity or placement might matter more?
- Is there any untried lever, such as bucket and probing structure or a different admission signal, that moves hit-rate more than noise?

## Continuation

Treat the cycle 3 v002 filter as the incumbent. Admission tuning (capacity, aging, counters, victim exemption) and insert-position tweaks are now exhausted on workload-a, so drop them. Don't stack admission filters on parents that already screen low-value inserts. Next cycle should try a structurally different lever (bucket and probing structure, or victim selection confirmed live). Alternatively it should validate the incumbent on other workloads and cache sizes. Add metadata-size and per-op-time to the fitness vector first, so noise-level winners like v001 can be rejected on cost.
