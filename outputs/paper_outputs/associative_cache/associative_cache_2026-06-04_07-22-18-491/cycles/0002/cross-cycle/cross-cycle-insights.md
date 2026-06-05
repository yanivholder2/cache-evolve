## Recurring wins

- Targeting the hot-item displacement problem pays off: both the LFU eviction change (Cycle 1 winner) and the second-chance stash (Cycle 2 winner via composition) address the same root failure — cold-item bursts permanently evicting hot items — and each produced a hit-rate gain with no regressions.
- Changes that exploit already-present structure win cheaply: the LFU counter was already carried on every `Item` at zero cost; activating it required minimal code churn and delivered immediate fitness improvement.
- Composed mechanisms reinforce rather than cancel: LFU eviction pre-filters stash entrants toward items with real reuse history, making stash promotions productive and allowing a small stash cap to suffice. The composition hypothesis from Cycle 1 was confirmed in Cycle 2.

## Recurring failures

- Eviction-scoring refinements (frequency counter cap, frequency-times-recency product score) measure as neutral on workload-a because the failure modes they address — counter inflation from stale-hot items, insertion-pressure edge cases, hot-set churn — do not occur on this trace. The logic may be sound but the workload does not trigger it.
- Memory-efficiency hypotheses are unverifiable: the `metadata-size` dimension has been absent from fitness vectors for two consecutive cycles, leaving structural costs (the stash's two `Vec` fields per bucket; any timestamp removal savings) unmeasured. Targeting memory efficiency while this gap persists is flying blind.

## Open questions

- Is `metadata-size` simply not wired into the evaluator for this subject, or is it a conditional dimension? This needs resolving before any memory-efficiency hypothesis can be graded.
- Would the counter-cap and product-score changes surface a benefit on a more adversarial trace with hot-set drift or bursty insertion patterns? Workload-a's stable hot set is a structural ceiling for any hypothesis that requires churn to activate.
- Is the stash size fixed at `bucket_size / 4` the right default, or would adaptive sizing (keyed to observed eviction rate or hit-rate feedback) recover additional hit rate?
- Do the LFU eviction and second-chance stash mechanisms saturate each other's gains at some point, or is there further head room as the stash policy is refined?
- The `timestamp` field removal from Cycle 1 v002 was structurally safe but did not win; is it worth landing as a clean-up baseline before any memory-efficiency hypothesis is retried?

## Per-policy notes

- **ClassicLRU (baseline):** Has clear slack in hit rate — two independent mechanisms both improved on it in Cycle 1 with no regressions, and their composition improved further in Cycle 2. Pure recency is insufficient for workload-a's access pattern.
- **LFU-augmented eviction (Cycle 1 v001 / Cycle 2 baseline):** Stable winner; activating the latent `lfu_counter` field was the minimal change that unlocked frequency-aware eviction. Now serves as the established baseline alongside the second-chance stash.
- **Second-chance stash (Cycle 1 v003 / Cycle 2 v001):** Effective as a complement to LFU eviction; the composition reveals that the stash is most productive when its entrant population is already pre-filtered by frequency. Structural memory cost (two `Vec` fields per bucket) remains unquantified.
- **Counter cap and product-score scoring (Cycle 2 v002, v003):** Produced no measurable change. Deprioritize further testing on workload-a unless a workload with hot-set drift is introduced.
