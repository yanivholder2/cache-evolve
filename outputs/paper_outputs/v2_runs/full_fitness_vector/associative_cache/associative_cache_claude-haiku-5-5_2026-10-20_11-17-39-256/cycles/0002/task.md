# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0002

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried three structural changes: wider buckets via the geometry layout, midpoint promotion of front-half hits, and bucket-local ghost memory for reinsertion. None of the three landed as a Winner. The one with a modest hit-rate gain (wider buckets) paid a small get-path cost, and the other two lost hit-rate on workload-a.

## Patterns

- Changes that move where items sit within a bucket (midpoint promotion, midpoint reinsertion) hurt hit-rate on this workload, and both add shifting work to the hot path.
- Widening buckets was the only change that improved hit-rate, but the gain was small, and the get path paid for the longer linear scan.
- Both bucket-local changes ran into the same suspected limit: their memory windows (back-half protection, the small ghost ring) may be too short to reach the reuse distances in workload-a.
- Put-side and get-side latency moved in ways the diffs do not fully explain in two variants. Put-side measurements should be re-verified before any variant is judged on them.
- Metadata-size did not register any change, so the size probe may be missing the new per-bucket state.

## Open questions

- Does the wider-bucket hit-rate gain hold across other traces, or is it specific to workload-a's conflict pattern?
- Is the ghost ring too small to matter, or is the reinsertion idea itself weak on this trace?
- What is driving the unexplained put and get latency shifts, and are they noise or layout effects?

## Continuation

Next cycle should keep the bucket-width direction and re-measure it before building on it. Drop midpoint promotion as currently specified. Before dropping ghost memory, test it with a longer ghost window or a hit-count gate, and make sure the size probe covers the new state.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
