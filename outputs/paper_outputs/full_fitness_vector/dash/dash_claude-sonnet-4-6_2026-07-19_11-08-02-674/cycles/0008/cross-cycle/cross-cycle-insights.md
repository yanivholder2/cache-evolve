## Recurring wins

- **MRU-end insertion** (Cycle 1, stable anchor ever since): flipping insertion from LRU-end to MRU-end improved hit-rate and per-op time simultaneously; the baseline's placement was actively working against the tier hierarchy's warm-up logic.
- **Struct field elimination** (Cycle 1): dropping dead-weight per-item metadata (e.g., `timestamp`) produces a dual dividend — smaller footprint and eliminated syscalls — with zero hit-rate penalty; the wins survive stacking with every subsequent mechanism.
- **Orthogonal mechanisms stack additively**: wins on non-overlapping subsystems (insertion position, struct layout, promotion gating, eviction selection) have reliably composed without interaction when applied to clearly separated code paths; stacking was empirically validated across Cycles 1→2 and 5→6.
- **Vacancy-gating as cascade prevention** (Cycle 3): constraining promotion fires to open slots eliminates cascade evictions by construction, delivering regression-free hit-rate lifts that threshold tuning alone cannot match.
- **Two-chance clock on Zipfian workloads** (Cycle 6): rotating warm items to MRU-end and stopping at the first counter-zero cold item terminates early where cold-tail items accumulate; this achieves frequency-aware eviction without an O(n) scan, improving all three measured dimensions simultaneously.

## Recurring failures

- **O(n) LFU-minimum victim scans**: every variant that scans a full bucket for the global minimum inflates put p99 consistently (Cycles 5, 6); the hit-rate gain is real but the cost is non-trivial and reproducible regardless of what else is combined with it.
- **Stash-tier interventions** (Cycles 2, 3, 4, 8 — bypass, cold eviction, frequency gates): the stash represents too small a fraction of capacity, and cascade-evicted items arriving there carry low frequency counts, so gate conditions fire too infrequently to shift the aggregate miss curve.
- **Raising frequency threshold above ≥ 2** (Cycle 4): real re-usable items sit in the 2–3 access range; stricter filtering ejects them, producing a net hit-rate loss rather than the expected gain.
- **Counter decay on a stationary workload** (Cycle 6): post-eviction counter halving suppresses legitimately earned promotion eligibility and adds a second O(n) scan; no tested scenario on the current trace benefits from aging out popularity.
- **Shared `lfu_counter` for clock warmth and promotion gating** (Cycle 7): the clock assumes counter-zero means cold; the promotion gate assumes non-zero means warm; using the same field for both purposes corrupts both signals and turns the clock's early-exit into O(k·n) rotation when the base is counter-rich.
- **Probing re-entry vacancy gate** (Cycles 3, 5, 7): structurally sound and always regression-free, but persistently thin — the joint condition (item at threshold AND target has vacancy) fires too rarely at high cache occupancy to produce a measurable hit-rate gain.

## Open questions

- Can a separate boolean "chance bit" — set only when an item is rescued from eviction, not on every access — decouple clock eviction from the frequency promotion counter, resolving the Cycle 7 conflict?
- Is there a sparser CLOCK_BIT condition (e.g., set only when `lfu_counter` crosses a second, higher boundary) that gives the clock enough discriminating power without over-marking warm items on every non-hottest access?
- Are there additional dead-weight struct fields beyond `timestamp` still present in the item layout that could be eliminated analogously to Cycle 1's discovery?
- Do eviction policy improvements (clock or LFU-min) interact differently at lower cache-to-working-set ratios, where cold-tail items are a smaller fraction of resident items and the clock's early-exit advantage shrinks?
- At what cache occupancy or workload shape do the stash-bypass and probing re-entry vacancy-gate mechanisms begin firing at a rate sufficient to produce measurable hit-rate changes?
- Are there promotion axes orthogonal to the frequency counter — for example, recency combined with frequency, or tier-residency duration — that could improve selectivity without a threshold knob?

## Per-policy notes

- **LRU (baseline)**: MRU-end insertion flip is the single highest-impact change found; baseline's LRU-end placement was its most costly structural choice and was corrected in Cycle 1.
- **LFU-minimum eviction**: confirmed independent hit-rate lift (Cycle 5); stacks with vacancy-gated promotion (Cycle 5); consistently inflates put p99 due to O(n) scan — the gain is real but the latency cost is structural and has not been eliminated by any variant tested.
- **Two-chance clock eviction**: best frequency-aware eviction policy found to date (Cycle 6); timestamp removal multiplies the gain by improving cache-line utilization during traversal; advantage degrades to O(k·n) when nearly all bucket items have non-zero counters (counter-rich base from promotion machinery), making decoupling of the warmth signal a prerequisite for composing it with frequency-gated promotion.
- **Vacancy-gated frequency promotion (≥ 2 threshold)**: reliable regression-free hit-rate lift; ≥ 2 appears near-optimal for this workload — raising to ≥ 3 caused a marginal net loss (Cycle 4); stacks cleanly with MRU-end insertion and struct reduction.
- **Counter decay**: harmful on a stationary popularity distribution; untested on a shifting workload — deprioritized until a non-stationary trace is available.
- **Stash cold-preferring eviction** (Cycle 8 v003): latency-safe because it fires only at full-bucket stash insertions rather than on every access; hit-rate neutral on current workload because the stash tier is too small; could become relevant on a workload where stash holds a larger share of total capacity.
