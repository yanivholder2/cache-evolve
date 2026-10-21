# Cycle 0006 Insight

## Cohort outcome

Three small policy tweaks on the cycle-0005 v003 parent were tried: a larger protected share (v001), demoted nodes reinserted at the LRU end of probation (v002), and `put` on an existing key no longer promoting (v003). All three were Winners, but the changes were tiny. Only v001 gained anything real, a small hit-rate improvement.

## Patterns

- The protected-share trend keeps improving hit-rate across steps, but each step gains less. The split is a weak lever with diminishing returns.
- Demoting a protected node to the next-to-evict position hurt hit-rate slightly. Demoted nodes have shown reuse and are better treated as warm, not cold.
- Skipping promotion on `put` updates left hit-rate unchanged. The trace rarely overwrites resident keys, so hit-rate is decided by the `get` and miss-insert paths.
- Metadata size was flat across the cohort, as expected for edits that touch no layout.
- Per-op-time deltas appeared even where hot paths were untouched (a constant-only edit, an untouched `get`). These are noise, which confirms the cycle-0005 suspicion that small latency deltas are not attributable. Winner status here likely reflects noise tolerance, not improvement.
- Authors could not read diffs, so attribution is inferred from hypotheses and fitness vectors.

## Open questions

- Where does the protected-share trend turn, if it does? Or is it saturating?
- Is there any hit-rate lever left in segment-boundary policy, or does the remaining gain lie elsewhere (admission, sizing, ghost history)?
- What is the actual per-op-time noise floor, and would a repeat-run measurement separate real deltas from noise?
- Does any of this hold on workloads other than the scan-heavy trace? This check has been pending for many cycles.

## Continuation

Drop `put`-update semantics, further demotion-position tweaks and more protected-share constant steps. They are exhausted or in the noise. Next, try a structurally different hit-rate lever, such as an admission filter or ghost-entry history for scan resistance, and use this cycle's v001 (larger protected share) as the baseline. Measure the noise floor and evaluate on non-scan workloads before crediting any speed claims.
