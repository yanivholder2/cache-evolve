# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three single-axis changes to the baseline were tried: sketch aging (v001), a small admission window (v002), and a sketch rewrite with a single hash, u8 counters and conservative update (v003). Only v003 won. It improved speed and sketch footprint with a small hit-rate gain.

## Patterns

- The two changes that added mechanism to the admission path (aging, window) gained nothing on hit rate and cost per-op time and struct metadata. The change that made the existing sketch cheaper and more accurate was the only one that paid off.
- Hit-rate hypotheses premised on "stale frequencies" or "newcomers need a trial period" did not hold on workload-a. The baseline's never-decaying counters and immediate strict-greater admission looked well suited to this trace.
- The baseline's per-op cost is dominated by repeated hashing in the sketch. Removing it gave the clearest gain, especially in the put tail.
- Added fields or bookkeeping on the hot path (halving counters, segment lookup on get) raised latency without a matching benefit.
- The metadata-size summary tracks Node size, not sketch heap, so v003's real 4x sketch saving was invisible in the headline number.

## Open questions

- Do the u8 saturation cap and conservative update still hold up on traces with larger frequency gaps or drift?
- Would aging help with a much longer period, or on a trace with real frequency drift? Would a larger window help?
- Which of v003's parts (single hash, u8 counters, conservative update) account for the gains?

## Continuation

Build on v003 as the new parent and push further on per-op cost and footprint (cheaper hashing, tighter sketch layout, less work on the hit path). Drop the naive aggressive aging and the ~1% window from v001 and v002. Retry them only in a materially different form, such as a much larger window, conservative aging or a doorkeeper, and only with a rationale tied to a specific trace property. Prefer hypotheses whose gains show in the headline dimensions.


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
