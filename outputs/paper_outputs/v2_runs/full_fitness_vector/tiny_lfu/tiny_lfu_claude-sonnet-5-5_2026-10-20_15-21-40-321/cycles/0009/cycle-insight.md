# Cycle 0009 Insight

## Cohort outcome

Three hypotheses were planned (adaptive window, relink-skip plus victim-estimate cache, flat open-addressing table with a slab and single hash). v001 was rejected: its implementer hit a backend "prompt too long" failure and produced nothing, so the adaptive window is still untested. Winners were v002 and v003. v003 gave a real, large latency gain. v002 was a regression that passed only on the selection rule.

## Patterns

- The only substantive win is a storage-layer change. Replacing the pointer-based map with a flat linear-probing table over a slab, hashing once per operation and reusing that hash for the sketch rows, cut get and put latency at median and tail. Hit-rate, Node size and policy stayed unchanged.
- This is the first blind hot-path guess in several cycles that paid off, and it contradicts the cycle-0008 worry that incumbents sit at the floor. The map and hashing path, not sketch, list or Node layout, was the dominant per-op cost on the baseline.
- v002 confirms that relinking and repeated victim estimates are not the cost. The epoch is bumped on nearly every access, so the victim cache almost never hits, and the bookkeeping slowed both paths and grew the cache struct. Hit-rate was unaffected.
- Hit-rate remains flat across every cohort, including a pure storage change. Metadata size is dominated by the sketch, so storage work cannot move it.
- v003's parent was the baseline, not the strongest hit-rate incumbent, so its gain is not yet shown to stack on the later lineages.

## Open questions

- Does the flat-table and single-hash gain carry onto the cycle-0004 v003 or cycle-0007 lineages? Cycle 0005 showed that a baseline-only speedup can fail to transfer.
- Is the v003 gain mostly from the flat table or from the single hash feeding the sketch? Earlier single-hash derivation failed to stack on strong parents.
- Does the adaptive window do anything for hit-rate on workload-a? It was never evaluated.

## Continuation

Port v003's slab and flat-table storage onto the best hit-rate incumbent, and separate the table's effect from the single-hash reuse, with repeat runs. Re-propose the adaptive hill-climbing window (H01) once, since it was never tested. Drop the victim-estimate cache and near-head relink skipping, and keep hypotheses aimed at the map and hashing path rather than the sketch or list.
