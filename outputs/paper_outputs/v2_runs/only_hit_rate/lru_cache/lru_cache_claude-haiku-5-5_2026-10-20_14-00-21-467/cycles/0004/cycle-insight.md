# Cycle 0004 Insight

## Cohort outcome

The cohort tried a hit-path lookup collapse (v001, named the Winner) and a probation/protected segmentation of the recency list that targets scan-heavy traffic (v003). v002 produced no insight file and was rejected for infrastructure reasons, so it tells us nothing about its approach. Only v003 reports a hit-rate movement, and it is not a Winner.

## Patterns

- The Winner's hit-rate equals its parent on workload-a, which fits a change meant to preserve eviction and recency order. Its stated target, per-op time on the hit path, was not measured, so the Winner rests on behaviour preservation alone. This is the same gap carried from cycles 0001 through 0003.
- v003 is the only variant this cycle with a reported hit-rate effect, but its insight gives no direction or magnitude, and it is not in the Winner list. Whether that reflects a regression, a smaller gain, or a selection-criterion problem cannot be read from the insights.
- v003's insight attributes its mechanism to H03. Cycle 0002 framed H03 as a policy-neutral per-op check, not a segmentation change. Either the hypothesis mapping or the mechanism label is off, and the lineage should be checked before reusing either.
- v003 adds a second list pointer and demotion logic, so it likely carries per-op and size costs that have not been measured.

## Open questions

- Does v003's hit-rate effect on workload-a go up or down against its parent, and by how much? Its insight does not say.
- Why was v001 chosen as the Winner over v003 if v003 is the only variant that moved hit-rate?
- What are the per-op time and metadata size costs of the probation/protected split, and does the hit-path collapse in v001 reduce per-op time at all?

## Continuation

Next cycle should measure per-op time and metadata size on v001 and on v003 before either lineage is extended, since no cohort has produced evidence on those dimensions. Re-check the selection criterion so that v003's hit-rate result is weighed on its own, and confirm the H03 mapping. Keep the segmentation idea alive as the one hit-rate-moving candidate, and treat v001's hit-path collapse as a cheap, eviction-neutral candidate that still needs a per-op measurement.
