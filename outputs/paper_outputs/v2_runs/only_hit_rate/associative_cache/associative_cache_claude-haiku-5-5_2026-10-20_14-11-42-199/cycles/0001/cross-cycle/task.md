# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried three structural changes: bucket repartitioning, an aging LFU policy, and a static-dispatch refactor of the bucket policy. Only v001 (fixed-width buckets) won, and its hit-rate gain was marginal. v002 and v003 showed no hit-rate movement and carried no measurements on other dimensions, so neither is judged yet.

## Patterns

- Hit rate was the only dimension measured in this cycle. Every Winner-or-not decision rests on it, and the gain that earned v001 its place was small.
- The latency and metadata-size effects the hypotheses predicted were not measured in any variant, so they are untested rather than refuted.
- Two of three variants left hit rate unchanged. For v002 this may mean the evaluation never selected the new policy; for v003 the refactor was meant to be neutral on hit rate, so no change is expected.
- Smaller bucket-conflict misses appear to be a minor share of total misses at this cache size, which limits how much further geometry changes can gain.

## Open questions

- Does the evaluation path actually select the aging policy in v002, or is the fixed aging period too coarse to alter eviction order?
- Does the per-operation cost of scanning wider buckets (v001) show up once per-op time is measured?
- Does the static-dispatch change in v003 reduce get latency when measured separately from put?

## Continuation

Next cycle should pursue changes whose effect can be checked on per-op time and metadata size, and include those dimensions in the fitness vector. Re-test the aging policy only after confirming the evaluation selects it. Drop further hit-rate-only geometry tweaks unless they are paired with a cost measurement.


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
