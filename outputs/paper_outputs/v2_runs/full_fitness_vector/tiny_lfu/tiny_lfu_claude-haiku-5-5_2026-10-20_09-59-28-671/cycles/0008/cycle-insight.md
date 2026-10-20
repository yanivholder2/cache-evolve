# Cycle 0008 Insight

## Cohort outcome

The cohort tried three changes aimed at frequency-sketch collisions: a conservative counter update that increments only the minimum-valued rows (v001), a put-path change that avoids re-reading the victim's frequency (v002), and reshaping the sketch from one fewer row to more columns at the same counter budget (v003). The two sketch-layout changes produced the only hit-rate movement, both small; the put-path change was flat on hit rate.

## Patterns

- Collision spread is the lever that moved hit rate. Both v001 (conservative update) and v003 (fewer rows, more columns) reduce how much colliding keys inflate each other's counts, and both showed small hit-rate gains on workload-a at size 2048. This is the first cycle since Cycle 0005 where a Winner gained hit rate.
- The gains carry a cost in different places. v001 adds an extra pass over a key's rows on every increment and shows slower get and put timings. v003's timing improvement is not explained by its stated change and may be noise or a layout effect.
- The put-path change (v002) left hit rate and metadata flat. Its only signal is a small put p99 shift from a single run, with a get p99 shift it cannot explain, so it is not a usable speed result yet.
- Metadata did not move in any variant. The v003 sketch stays within the baseline memory budget.
- Every result comes from one run on one trace and one cache size. Source overlays were not inspected in this cycle, so mechanisms are inferred from hypotheses and fitness results, and the v003 row and column counts are not recorded.

## Open questions

- Does the v003 row-for-column trade hold on a second trace and at other cache sizes, or is the gain specific to workload-a at size 2048?
- Can the conservative update in v001 keep its collision benefit without the extra per-increment pass over all rows?
- Is the v003 timing gain real, or is it measurement noise?
- Does the put-path victim-read change in v002 show a repeatable put p50 or p99 gain across at least three runs?

## Continuation

Pursue the sketch-collision direction: the row-versus-column trade from v003 is the stronger candidate, so confirm its hit-rate gain on another trace and size before building on it. Hold the conservative update from v001 as a hit-rate option only if it can be made without the extra per-row pass. Drop the put-path victim-read change (v002) as a speed direction until repeat put timings show a consistent gain.
