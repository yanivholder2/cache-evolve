## Recurring wins

- CLOCK-family variants keep landing on the Pareto front when they encode recency-leaning per-slot state (promotion-on-second-touch, ghost-ring memory of just-evicted keys). Two cycles in a row, the surviving variant has been a CLOCK refinement.
- Layering complementary mechanisms (e.g. promotion class + small ghost ring) beats widening a single existing signal — distinct gaps get closed at once rather than one being over-tuned.
- The baseline arena/slot-reuse scaffolding has tolerated large representational rewrites cleanly across both cycles; structural invariants (P1/P2/P3) are not the bottleneck.

## Recurring failures

- SLRU rebuilds have been attempted twice and remain uninterpretable both times: variant and baseline were sampled at disjoint capacities, so no same-size delta over LRU has actually been demonstrated.
- Frequency-leaning per-slot bits (saturating aging counters, GCLOCK-style) regress on recency-skewed traces by shielding stale-popular keys and cold-scan inserts.
- Variants that bet on dimensions the fitness vector does not score (per-op time, metadata size) produce real structural wins that are invisible to selection and risk being discarded as neutral noise.

## Open questions

- The fitness vector itself remains the unaddressed meta-bottleneck: until per-op-time and metadata-size dimensions are scored, time- and space-targeted variants cannot be selected on their actual merit.
- Are CLOCK-Pro-lite-style wins specific to the currently sampled recency-skewed trace, or do they generalize to workloads with different re-reference-distance profiles?
- Can SLRU ever be evaluated honestly without first fixing sampling discipline so parent and child share capacities?
- What is the right ghost-ring sizing and Hot-demotion aggressiveness for the current champion, and how far can it be pushed before re-introducing the frequency-bit trap?

## Per-policy notes

- LRU (baseline): structurally permissive — accommodates SLRU rebuilds, CLOCK rewrites, and link-representation changes without breaking invariants. Has not been beaten on its own terms in a matched-size comparison.
- CLOCK: the productive lineage so far. Plain CLOCK landed on the front in cycle 1; CLOCK-Pro-lite (cold/hot promotion + ghost ring) extended the win in cycle 2 and is the current working champion.
- GCLOCK / frequency-counter CLOCK extensions: actively harmful on the currently scored trace; drop the lineage until other workloads are scored.
- SLRU: structurally clean but evidentially stuck — two attempts, zero interpretable results due to capacity-mismatch sampling. Do not queue again without guaranteed matched capacities.
- Pure representation-only repacks (e.g. u32+sentinel link encoding): on hold until a metadata-size dimension exists to score them.
