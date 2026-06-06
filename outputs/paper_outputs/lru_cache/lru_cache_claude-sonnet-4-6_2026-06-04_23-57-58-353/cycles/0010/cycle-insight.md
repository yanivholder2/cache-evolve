# Cycle 0010 Insight

## Cohort outcome

The cohort probed two independent threads from cycle 9: whether ghost admission complements frequency-primary eviction (regardless of tie-breaker), and whether staleness outperforms IRR as the secondary signal. Restoring ghost admission (v002) and switching to staleness (v003) both produced gains; the IRR-plus-ghost combination (v001) regressed due to a structural conflict between the two mechanisms, yet v001 advances because its high-scoring parent line still places it among the cycle's best absolute scores.

## Patterns

- Ghost admission and frequency-primary eviction are confirmed complementary and structurally orthogonal: ghost acts at insertion time to shortcut warm re-arrivals into protected, while frequency ranks eviction candidates independently. Restoring ghost to the cycle-9 design recovered the regression cycle 9 introduced by removing it, closing the loop on that thread.
- IRR is actively harmful in the ghost-admission context. Ghost-admitted entries re-enter protected with no second timestamp, making their IRR score the maximum sentinel — indistinguishable from the coldest possible newcomer. This causes the IRR tie-breaker to undermine exactly the entries ghost admission is designed to favor, producing a net regression when the two mechanisms are combined.
- Staleness avoids IRR's sentinel blind spot: because it requires only a single prior access to be finite, it correctly ranks the single-access entry population — the most common cold-entry class — against each other and against stale-burst entries at the same frequency level. The improvement is consistent with cycle-9's staleness-product validation on the IRR baseline.
- The pattern from cycles 9 and 10 together: frequency is the right primary signal; ghost is a net-positive insertion shortcut; staleness is a better tie-breaker than IRR on this workload because it is defined from the first access rather than from the second.

## Open questions

- Does staleness as the secondary signal in the ghost-admission context yield an additive gain — or does the ghost population (entries with prev_access=0 in IRR terms, but finite and growing staleness) change the tie-breaker dynamics in unexpected ways?
- Is there a cohort of workloads where IRR's richer two-timestamp signal outperforms staleness despite the single-access blind spot, or is staleness strictly superior on workloads with high cold-entry churn?
- With ghost admission confirmed positive and staleness confirmed as a better secondary than IRR, is the current design (frequency primary, staleness secondary, ghost admission, 80/20 split, fixed scans) at a local optimum, or are there structural changes — such as adaptive ghost capacity — that could further improve it?

## Continuation

Next cycle's clearest hypothesis is frequency primary plus staleness tie-breaker plus ghost admission: combining the signal improvement validated by v003 with the structural shortcut validated by v002, and confirming they are additive. This is the natural completion of what cycles 9 and 10 validated separately. IRR should be dropped as a tie-breaker in any design that includes ghost admission — the interaction is harmful and the mechanism is not recoverable without special-casing ghost entries at eviction time. Secondary priorities are multi-workload evaluation (the staleness gain has now been seen on two baselines but only one trace) and testing whether ghost capacity scaling (relative to the active working-set size) changes the magnitude of the ghost benefit.
