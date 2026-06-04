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
