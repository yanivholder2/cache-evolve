# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Two structural variants were evaluated: a window plus segmented main region (v002) and a compact u8 sketch with single-hash indexing and conservative update (v003). The aging variant (H01, v001) was rejected at fitness because the simulator build failed on an HTTP 429 while downloading the Caffeine jar, so it never produced a result. v002 won with a marginal hit-rate gain, and v003 was essentially neutral.

## Patterns

- Both evaluated variants moved hit-rate only negligibly on workload-a, so the baseline's sketch-gated admission already captures most of the available benefit.
- Both fitness vectors reported hit-rate only. The metadata-size and per-op-time effects, which were the stated goal of H03 and the main risk of H02, are unmeasured.
- Narrower counters and cheaper hashing cost no accuracy, so the sketch looks over-provisioned.
- The only rejection was infrastructure-related (network rate limit), not a flaw in the idea. Aging remains untested.

## Open questions

- Does periodic halving (aging) help? It was never evaluated.
- Do the v002 window/segmentation and v003 sketch compaction help or hurt on metadata size and per-op time?
- Is the small v002 win real, or noise? Other traces or cache sizes might show a different result.

## Continuation

Re-attempt aging (H01) first, since the simulator failure was environmental, and check that the simulator's dependencies are cached or retried before the run. Make sure the fitness output includes metadata-size and per-op-time, so v002 and v003 can be judged on cost. Consider combining v003's compact sketch with aging or the window. Don't keep tuning hit-rate on structure alone until the missing dimensions are measured.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
