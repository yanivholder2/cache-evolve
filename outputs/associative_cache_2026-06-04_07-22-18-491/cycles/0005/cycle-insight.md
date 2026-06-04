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
