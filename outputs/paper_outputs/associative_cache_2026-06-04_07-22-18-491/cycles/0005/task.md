# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0005

## Winners

- v003

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0005/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0005/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0005/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0005/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0005/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants probed the promotion-credit mechanism and stash intake policy, and all three regressed hit rate — making this a bounding cycle rather than an advancing one. The cohort collectively closed off the remaining open questions about the promotion-credit lever without producing a new baseline improvement.

## Patterns

- The promotion-credit axis is now fully characterised: flat +3 overshoots the one-burst-survival threshold, and frequency-proportional scaling up to +4 amplifies the same failure in a more nuanced form. Both regressions confirm +2 as the local optimum, not merely a promising candidate.
- The frequency-proportional credit and the stash intake filter share a root misconception: both treat the item's LFU counter at eviction time as a proxy for future demand. On a Zipfian workload, past counter value reflects burst-displacement timing far more than genuine reuse probability, making counter-derived heuristics at the promotion boundary unreliable.
- The stash-filter rejection establishes that zero-counter items at eviction time are not cold waste — they are frequently just-inserted items displaced before the workload could access them again. Stash value depends on holding all recent evictions, not a quality-filtered subset.

## Open questions

- The `metadata-size` measurement gap has now persisted four cycles without resolution. The stash adds per-bucket `Vec` fields whose memory cost remains entirely invisible to the fitness function.
- Are there structural improvements outside the stash-and-promotion subsystem that have gone unexplored because three consecutive cycles have converged on the same mechanism?
- Would any of the rejected hypotheses in this cycle show a different outcome on a workload with non-stable hot-set membership, where past counter values more reliably predict future access?

## Continuation

The promotion-credit lever and stash intake filter are exhausted as hypothesis axes — next cycle should leave both fixed at the Cycle 3 optimum (+2 credit, no intake filter, FIFO stash, bucket-quarter capacity). The highest-leverage unresolved thread is the `metadata-size` measurement gap: four cycles of stash variants have accumulated unquantified per-bucket memory overhead; if the evaluator can surface this dimension, a deliberate memory-versus-hit-rate trade-off variant becomes the most verifiable new direction. If the gap cannot be closed, the next productive structural axis lies outside the stash mechanism entirely.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
