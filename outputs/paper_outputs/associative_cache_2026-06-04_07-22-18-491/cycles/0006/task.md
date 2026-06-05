# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0006

## Winners

- v001

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0006/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0006/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0006/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0006/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/associative_cache_2026-06-04_07-22-18-491/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle explored three structurally distinct directions: removing dead metadata from the item struct to surface memory overhead, extending eviction memory one tier further via a key-only ghost cache, and rethinking the admission boundary by routing all new inserts through a probationary probe zone before primary. Only the admission-control variant won, and it branched from baseline rather than from the accumulated stash-plus-LFU chain.

## Patterns

- Probationary admission is a stronger lever than any eviction-side refinement tested so far. By silently discarding one-time-access items in the probe FIFO before they ever compete for a primary slot, v003 prevented the cold-insert churn that ClassicLRU could not distinguish from hot traffic — a gain large enough to win outright from a simpler baseline.
- The ghost tier confirmed the same cold-signal principle that the stash-intake filter established in Cycle 4: items that age through primary and stash without a single re-access are reliably cold, not unlucky hot items. Giving them a frequency head start on re-insertion inverts the LFU signal and costs hit rate; extending eviction memory further back in history adds noise on this workload.
- The metadata-size evaluator gap is now fully confirmed as a pipeline infrastructure problem, not a variant problem. v001 removed the dead `timestamp` field and the structural improvement compiled correctly, but the `evolve_metadata_size` binary is not wired into the evaluation pipeline, so the fitness vector never reflected the change. Four-plus cycles of memory-efficiency hypotheses remain unverifiable until this is resolved at the harness level.
- The winning approach descended from baseline, bypassing four cycles of stash-mechanism accumulation. This does not invalidate the stash; it opens the question of whether admission control and the stash are complementary.

## Open questions

- Can probationary admission and the stash-plus-LFU-plus-promotion-credit mechanism be composed? H03 explicitly anticipated this; the two approaches target different failure modes (spurious admission vs. premature eviction of re-accessed items).
- Is the half-capacity probe zone the right sizing, or does the gain depend on the ratio of probe-to-primary capacity in a non-obvious way?
- The `metadata-size` measurement gap has now been confirmed rather than merely suspected — is it addressable at the evaluator configuration level, or does it require harness changes?

## Continuation

Next cycle should branch from v003 and explore composing probationary admission with the stash mechanism: add the Cycle 3 stash (LFU eviction, FIFO stash at quarter capacity, +2 promotion credit) onto the probe-zone baseline to test whether the two approaches reinforce each other. A second hypothesis worth testing is probe zone sizing — varying the probe cap above and below half-capacity to characterise the gain curve. The ghost tier and promotion-credit axes are closed. The metadata-size gap is a confirmed evaluator infrastructure gap; variants targeting memory efficiency should be deferred until it is fixed at the harness level.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
