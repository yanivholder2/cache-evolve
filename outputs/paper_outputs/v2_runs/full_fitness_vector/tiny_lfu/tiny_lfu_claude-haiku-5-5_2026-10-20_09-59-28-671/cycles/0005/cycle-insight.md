# Cycle 0005 Insight

## Cohort outcome

The cohort tried three access-pattern changes: splitting the put-side unlink and promotion path into a separate helper (v001), moving the per-row sketch seeds from a per-instance field into a module-level constant table (v002), and restoring the public per-row seeded indexing on top of a single precomputed hash (v003). All three are Winners, but for different reasons: v001 and v002 are per-op-time gains with hit rate unchanged, and v003 is a hit-rate correction that gives back speed.

## Patterns

- Seed storage was the cost behind the earlier size and speed regressions. Keeping the seeds in a `const` table, not per instance, removed the struct growth and kept the speedup, with hit rate unchanged.
- Index fidelity and speed trade off directly. Matching the public per-row seeded mixing restored the admission decisions, but reintroduced per-row work on both get and put. The speed win and the correct indices cannot both be kept by the current formulation.
- Put-side latency responds to the shared unlink and promotion path. Splitting it out improved put timing, but this rests on a single recorded run and has not been separated from noise.
- Every Winner kept hit rate at or near its parent's value except the index correction, and none of the variants changed the metadata reported for the policy itself. Source diffs were not inspected, so the mechanisms are inferred from hypotheses and fitness results.

## Open questions

- Can the public per-row indices be produced with less per-row work than the seeded mixing function, so the fidelity fix does not cost the speed gain?
- Do the `const`-table indices match the per-access derivation for every key, or only in aggregate hit rate?
- Does the put-side improvement from v001 survive repeat runs, and does it hold on a second trace?

## Continuation

Pursue the `const`-table seed layout from v002 and verify its index equivalence against the public path before building further on it. Keep the index-correctness direction from v003, but make the per-row derivation cheaper rather than accepting the speed loss. Hold the put-path split from v001 as a candidate until repeat runs confirm the gain.
