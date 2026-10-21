# Cycle 0009 Insight

## Cohort outcome

Three small tweaks to the one-line aged doorkeeper incumbent: a lower aging threshold (v001), a single-probe filter with a higher threshold (v002), and an early return in put for resident keys (v003). All were Winners, but none improved latency. Only v001 gained hit-rate, and the gain was marginal. v002 lost a little hit-rate, and v003 held hit-rate flat. Metadata stayed flat in all three.

## Patterns

- Per-op-time got worse in all three, including get latency where get was not meant to change. This is now the seventh cycle in a row in which an edit to the hot path or the admission filter regressed latency. Layout effects or noise are the most likely cause.
- The doorkeeper is tuned out. Aging threshold and probe count are weak levers with only small hit-rate trade-offs. Once both probes share a cache line, the second probe costs almost nothing, so dropping it saves no time and weakens selectivity.
- Skipping filter bookkeeping on resident-key puts bought nothing, so that bookkeeping is not a meaningful cost.
- Winner status is again weak evidence, since flat hit-rate and metadata are enough to qualify.
- Variant authors for v002 and v003 could not read the source, so the mechanisms are inferred from fitness vectors.

## Open questions

- What is the run-to-run noise floor for get and put latency on the unchanged incumbent? This has now been asked for six cycles and is still unanswered.
- Is v001's small hit-rate gain real or within noise?
- Would changes to eviction or promotion, rather than admission, move hit-rate further?
- Do these conclusions hold on a second trace with scans or frequency skew?

## Continuation

Stop tuning the doorkeeper (aging threshold, probe count) and stop editing put or get control flow for latency. Next cycle should first measure the noise floor with repeated baseline runs, comparing against the parent within the same run. After that, aim hypotheses at eviction or promotion policy, and check them on a second trace. Do not credit any latency change without repeats.
