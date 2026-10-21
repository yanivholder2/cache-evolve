# Cycle 0009 Insight

## Cohort outcome

Three probes were tried: a second halving of the ghost table (v001), in-place reuse of the evicted node's slot in `put` (v002), and ghost-hit keys inserted at probation MRU or second position (v003). All three were Winners, but none showed a verified gain. v001 held hit-rate level, v002 improved nothing, and v003 lowered hit-rate. Winner status looks like noise tolerance.

## Patterns

- The ghost table still has headroom. A second halving did not cost hit-rate, but put tail latency did not improve either, so shrinking it no longer buys anything measurable.
- Cutting the allocate/free pair in `put` (v002) did not help. Map remove and insert dominate put cost, so node allocation is not the lever.
- A middle path for ghost hits (v003) hurt hit-rate. The ghost filter helps by keeping suspicious keys cheap to evict, not by rewarding remembered ones. Both ghost-hit promotion routes (protected last cycle, probation MRU now) have failed.
- `get` timing moved in all three variants, mostly at the tail, although `get` was never edited. Run-to-run or layout noise swamps small per-op-time deltas, as in every cycle since 0005.
- The metadata-size metric cannot see heap-allocated ghost memory, so the memory saving from ghost shrinking remains unmeasured.
- Authors could not read the code, so attribution rests on hypotheses and fitness vectors.

## Open questions

- What is the per-op-time noise floor? Without a repeat-run measure, no latency claim can be credited or blamed.
- Does the ghost-gated design generalize to non-scan workloads? This has been pending for many cycles.
- Can map operations per put be reduced structurally, for example with a single combined lookup/insert (entry-style API) or a cheaper map?
- Is the remaining hit-rate gain in admission or sizing, since insertion-position and segment-share levers are exhausted?

## Continuation

Treat cycle-0008's ghost-gated baseline, possibly with the smaller ghost table, as the parent. Drop ghost-hit insertion position, ghost shrinking, node-slot reuse, and protected-share steps. Next cycle, build a noise-floor and heap-inclusive memory measurement, and evaluate on non-scan workloads. Then try a structurally different lever: fewer map operations per put, or an admission policy.
