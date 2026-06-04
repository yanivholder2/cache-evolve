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


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle tested three hypotheses branching from Cycle 1's winner: composing LFU eviction with a second-chance stash (the composition hypothesis recommended by Cycle 1), capping the frequency counter to prevent stale-hot items from blocking rising ones, and replacing the lexicographic eviction criterion with a frequency-times-recency product score. Only the composition variant won, improving hit rate; the other two produced no measurable change in either direction.

## Patterns

- The composition hypothesis was confirmed: LFU eviction and the second-chance stash are mutually reinforcing rather than redundant. LFU pre-filters the stash population toward items with real reuse history, so stash promotions are productive far more often than a naive second-chance mechanism would be, and a small stash cap proves sufficient.
- Both neutral variants (counter cap, product score) failed for the same underlying reason — workload-a's hot set is stable enough that the failure modes they targeted (counter inflation blocking rising items; newly inserted items being immediately the weakest LFU candidate) simply do not occur on this trace. Neither change is harmful; neither is triggered.
- Workload-a's stability is becoming a structural ceiling: any hypothesis that requires hot-set churn or insertion-pressure edge cases to show a benefit will measure as neutral on this trace regardless of how sound the logic is.

## Open questions

- Are the counter-cap and product-score improvements real but latent — would they surface on a more adversarial trace with hot-set drift or bursty insertion patterns?
- The `metadata-size` measurement gap flagged in Cycle 1 remains unresolved: memory overhead of the stash (two `Vec` fields per bucket in the winning variant) has still not been quantified.
- Is stash sizing fixed at `bucket_size / 4` the right default, or would adaptive sizing (e.g. keyed to observed eviction rate) recover more hit rate?

## Continuation

The composition of LFU eviction plus second-chance stash is now the established new baseline; next-cycle hypotheses should branch from this cycle's v001. Eviction-scoring refinements (counter caps, alternative scoring functions) should be deprioritized unless a more adversarial workload is introduced that can actually surface hot-set churn — testing them further on workload-a will continue to read as neutral. The unresolved `metadata-size` dimension should be investigated: if the evaluator cannot report memory overhead, the stash's structural cost is invisible and variants targeting memory efficiency remain unverifiable.


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
