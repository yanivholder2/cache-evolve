## Recurring wins

- **Admission-boundary interventions outperform eviction-side tuning.** The largest single-cycle gains have come from changing what enters primary (probationary probe zone, Cycle 5) rather than how primary evicts. Admission control discards one-time-access items before they ever displace residents.
- **Exploiting latent data-model capacity wins immediately.** LFU eviction (Cycle 1) succeeded because the frequency counter was already carried on every item at zero marginal cost; structural changes with no new overhead convert into hit-rate gains without regressions.
- **Composing complementary mechanisms reinforces rather than saturates.** LFU eviction reduced which cold items reached the stash; the stash recovered items that slipped through primary anyway. Composition (Cycle 2) produced additive gain, not diminishing returns.
- **FIFO ordering inside the stash beats frequency-based ordering.** Every item in the stash arrived because LFU already identified it as lowest-frequency in primary; the within-stash frequency gradient is too compressed to discriminate. FIFO's age ordering is better matched to the stash's purpose.
- **Promotion credit fixes round-trips at the stash-to-primary interface.** Raising the credit from +1 to +2 (Cycle 3) cleared the tie that caused freshly promoted items to be re-evicted immediately on the next cold burst, converting churn into genuine secondary hits.

## Recurring failures

- **Eviction-scoring refinements do not register on a stable hot-set workload.** Counter caps and frequency-times-recency product scores (Cycle 2) both measured neutral because workload-a's hot set is stable enough that counter inflation and insertion-pressure edge cases never occur on this trace. The logic is sound but the trigger conditions are absent.
- **Extending eviction memory further back into history adds noise.** The ghost cache (Cycle 5) gave re-inserted cold items a frequency head start that inverted the LFU signal and cost hit rate. Items that aged through primary and stash without re-access are reliably cold, not unlucky hot items.
- **Stash capacity expansion accumulates noise rather than useful survivors.** Doubling the stash cap (Cycle 3) let cold items linger long enough to earn spurious promotions that displaced medium-frequency primary residents. The small FIFO cap is a self-cleaning property, not a constraint.
- **Stash intake filtering by eviction-time counter misreads the workload.** Zero-counter items at eviction time are frequently just-inserted items displaced before the workload could access them again, not inherently cold waste. Rejecting them removes legitimate stash candidates.
- **Promotion credit above +2 overshoots.** Flat +3 and frequency-proportional scaling up to +4 (Cycle 4) both regressed; the threshold is the one-burst-survival boundary, not a gradient. Counter-derived heuristics at the promotion boundary are unreliable on Zipfian traces because past counter value reflects burst-displacement timing more than genuine reuse probability.

## Open questions

- **`metadata-size` measurement gap is a confirmed harness infrastructure problem.** Five cycles of stash and memory-efficiency variants have run with no quantification of per-bucket memory overhead. The `evolve_metadata_size` binary is not wired into the evaluation pipeline; variants targeting memory trade-offs remain unverifiable until this is fixed at the harness level.
- **Do probationary admission and the stash-plus-LFU chain compose?** The two mechanisms target different failure modes — spurious admission versus premature eviction of re-accessed items — and may reinforce each other the same way LFU and the stash did in Cycle 2.
- **What is the optimal probe zone size?** The half-capacity probe cap won in Cycle 5, but the gain curve above and below that ratio has not been characterised.
- **Would workload-sensitive hypotheses show gains on non-stable traces?** Counter caps, product-score eviction, ghost caches, and stash intake filters all have coherent failure modes as their motivation; they have only been tested on a workload where those failure modes do not occur.
- **Does dynamic promotion credit outperform fixed +2?** A repeat-promotion multiplier (scaling credit by how many times an item has cycled through the stash) remains untested.

## Per-policy notes

- **ClassicLRU (original baseline):** Cannot distinguish cold-insert churn from hot traffic; hot items are permanently displaced by cold-item bursts. Served as the branching point for Cycle 5's admission-control variant, which won directly from this baseline.
- **LFU eviction (Cycle 1 winner, new baseline from Cycle 2):** Activates the latent frequency counter; reduces cold-item displacement with no new structural overhead. Became the foundation for two subsequent cycle winners.
- **LFU eviction + FIFO second-chance stash + +2 promotion credit (Cycle 2–3 accumulated baseline):** The established compound baseline after Cycle 3. Stash cap at quarter bucket capacity; promotion credit fixed at +2. All further stash-internal and promotion-credit tuning regressed in Cycle 4; this configuration is the stash mechanism's local optimum on this workload.
- **Probationary admission / probe zone (Cycle 5 winner):** Largest gain of any single cycle, and it branched from ClassicLRU rather than the accumulated stash chain. Silently discards single-access items in a probe FIFO before they compete for primary slots. Composition with the stash chain is the highest-priority unresolved hypothesis.
