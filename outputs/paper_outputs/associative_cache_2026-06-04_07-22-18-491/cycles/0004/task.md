# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004

## Winners

- v001
- v002

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0004/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants tuned parameters of the LFU-plus-stash baseline: stash capacity, eviction policy within the stash, and the promotion credit awarded when a stash item re-enters primary. Two of the three regressed hit rate; only the promotion-credit increase won.

## Patterns

- The stash's small FIFO cap is a self-cleaning mechanism, not a constraint. Doubling the cap let cold items linger long enough to earn spurious promotions that displaced genuinely medium-frequency residents — larger capacity accumulated noise rather than useful survivors.
- Applying LFU selection inside the stash is redundant: every item in the stash arrived there because LFU already identified it as the lowest-frequency item in primary, so the frequency gradient within the stash is too compressed to discriminate. FIFO's age-based ordering is better matched to the stash's actual role.
- The winning change operated at a different structural boundary — the transition back into primary — rather than inside the stash itself. Raising the promotion credit from +1 to +2 cleared the tie that caused promoted items to be immediately re-evicted on the next cold burst, converting round-trips into genuine secondary hits.
- Both regressions and the win together suggest the stash mechanism is now well-calibrated in its internal design; the remaining slack is at the interface between stash and primary.

## Open questions

- Is +2 the right promotion credit, or does the gain plateau or reverse at higher values? The win was described as small, implying the lever has a range that has not been fully explored.
- Would dynamic promotion credit — scaled by how many times an item has cycled through the stash — outperform a fixed value?
- The `metadata-size` measurement gap persists after three cycles of stash variants; the stash's per-bucket memory overhead remains unquantified.

## Continuation

Next cycle should branch from v003 and explore the promotion-credit lever further: test +3 and/or a simple repeat-promotion multiplier to determine whether the gain is monotone or peaks near +2. Stash-cap and stash-eviction-policy changes are confirmed suboptimal on this workload and should be dropped. The `metadata-size` gap is now a three-cycle-old blind spot — if the evaluator can report it, a variant that deliberately trades memory for hit rate (or vice versa) would finally be verifiable.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
