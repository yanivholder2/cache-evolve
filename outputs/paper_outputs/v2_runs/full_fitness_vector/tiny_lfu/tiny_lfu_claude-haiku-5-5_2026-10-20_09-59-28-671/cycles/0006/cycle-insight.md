# Cycle 0006 Insight

## Cohort outcome

The cohort tried three changes to the sketch's indexing and the protected segment: premixing the per-row seeds into a module-level `const` table (v001), demoting the protected tail to probation MRU (v002), and grouping a key's sketch counters into one cache-line block (v003). Each Winner is a single-axis trade-off rather than a clean net gain: v002 showed a timing gain with hit rate unchanged, v003 showed a clear per-op speedup at a cost in hit rate, and v001 kept hit rate and metadata flat while its timing read as a regression.

## Patterns

- Per-op time is again the only axis that moved decisively, and it moved in both directions. The block-grouped layout in v003 gave the clearest speedup, but it changed which keys collide in the sketch and cost hit rate. Index changes that alter collisions look like the main source of hit-rate movement this cycle.
- Premixed seed tables kept hit rate and metadata unchanged, which fits the claim that counter indices are preserved. They did not make the per-row path faster, and the timing signal from the single run was unstable, so the Cycle 0005 `const`-table layout should not be treated as a speed win yet.
- Protected-tail demotion produced no hit-rate change at all. Either the protected segment never reached its capacity limit on this trace, or demoted entries were not re-referenced. This is a test-coverage question as much as a result.
- Several attributions remain unverified. Source diffs were not inspected, the timing results come from single runs, and all evidence comes from one trace (workload-a). One metadata movement in v003 is not explained by the hypothesis.

## Open questions

- Can the block-grouped counter layout from v003 keep its per-op gain while preserving the original collision pattern, or does the speedup require the collision change?
- Does the protected-capacity branch fire at all on workload-a? If not, the demotion hypothesis was untested rather than refuted.
- Do the timing shifts in v001 and v003 survive repeat runs, and do any of these hit-rate effects hold on a second trace?
- What caused the reported metadata growth in v003 when the hypothesis forbade struct changes?

## Continuation

Pursue the cache-line grouping from v003, but keep the speed gain and restore collision-preserving block selection, or pair it with a compensating counter-width or aging change, before accepting it. Do not spend another cycle on protected-segment turnover until it is confirmed the branch fires on this trace. Hold the seed `const`-table premix from v001 as a neutral layout option, not a speed direction, until repeat runs settle its timing.
