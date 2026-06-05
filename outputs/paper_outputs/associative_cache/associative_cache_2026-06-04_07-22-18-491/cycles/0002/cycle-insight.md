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
