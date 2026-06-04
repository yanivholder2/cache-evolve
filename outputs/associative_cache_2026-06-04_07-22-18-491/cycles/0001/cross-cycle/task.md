# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants explored the baseline `ClassicLRU` eviction path from different angles — frequency weighting, dead-field removal, and a second-chance stash — and the sole winner was the frequency-aware eviction change, which improved hit rate with no regressions.

## Patterns

- Both hit-rate improvements (frequency-aware eviction in v001, second-chance stash in v003) came from complementary mechanisms targeting the same root failure: hot items being permanently displaced by cold-item bursts. Neither hurt any measured dimension, suggesting the baseline has slack in both directions.
- v001 won by activating already-latent capacity in the data model (`lfu_counter` was already carried on every `Item` at zero cost), making the change structurally minimal with immediate fitness payoff.
- The `metadata-size` dimension was absent from the fitness vectors of both v002 and v003, leaving the memory cost of the stash (v003) and the savings from removing `timestamp` (v002) entirely unverified. This is a systematic measurement gap, not an isolated miss.

## Open questions

- Is `metadata-size` simply not wired into the evaluator for this subject, or is it a conditional dimension that only appears when a variant explicitly targets it?
- v003 showed a clear hit-rate gain but its stash adds two `Vec` fields per bucket — does that memory overhead offset or exceed the benefit once measured?
- Do v001 and v003 compose? Frequency-aware eviction reduces which items enter the stash while the stash recovers items that slip through — the gains may compound.

## Continuation

Next cycle should test composition: apply v001's LFU eviction on top of v003's second-chance stash to probe whether the two mechanisms reinforce each other or saturate the same gain. In parallel, the `metadata-size` measurement gap should be investigated — if the evaluator cannot report it, variants targeting memory efficiency are flying blind. The `timestamp` removal from v002 is structurally safe and worth landing as a clean-up baseline before any memory-efficiency hypothesis is re-attempted.


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
