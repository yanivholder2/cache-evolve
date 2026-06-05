## Recurring wins

- **Layered complementary mechanisms**: pairing LFU eviction with a second-chance stash produced a compounding hit-rate gain — not because either mechanism alone was revolutionary, but because LFU pre-filters the stash population so that stash promotions are productive rather than noisy. The combination was confirmed across two consecutive cycles.
- **Low-structural-cost changes pay off fastest**: the first winner (frequency-aware eviction) required no new fields — `lfu_counter` was already carried on every item at zero cost. Changes that activate latent capacity in the existing data model have consistently delivered immediate fitness gains with no regressions.
- **Interface-boundary tuning outperforms internal-mechanism tuning**: when the stash's internal design (capacity, eviction order) was already well-calibrated, the remaining gain came from adjusting the transition back into primary (promotion credit). Improvements at structural interfaces have been more durable than improvements inside a single component.

## Recurring failures

- **Eviction-scoring refinements on stable workload-a**: both the frequency counter cap and the frequency-times-recency product score measured as neutral. The root cause is the same each time — workload-a's hot set is stable enough that the edge cases these changes target (counter inflation, insertion pressure) do not occur. Continuing to test scoring variants against this trace will keep reading neutral regardless of theoretical merit.
- **Enlarging the stash capacity**: doubling the stash cap regressed hit rate by letting cold items linger long enough to earn spurious promotions that displaced genuine medium-frequency residents. More stash room accumulates noise, not useful survivors.
- **LFU ordering inside the stash**: applying LFU selection within the stash was redundant and harmful — every item there already arrived via LFU from primary, so the frequency gradient inside the stash is too compressed to discriminate. FIFO's age-based ordering is better matched to the stash's role.

## Open questions

- **`metadata-size` measurement gap**: three cycles of stash variants have shipped with no quantification of per-bucket memory overhead. If the evaluator cannot report this dimension, variants trading memory for hit rate (or vice versa) are unverifiable and any memory-efficiency hypothesis is flying blind.
- **Promotion-credit optimum**: the +2 win in cycle 3 was described as small. Is the gain monotone beyond +2, does it plateau, or does it reverse? A +3 probe and/or a repeat-promotion multiplier are the natural next step to characterise the lever's range.
- **Dynamic vs. fixed promotion credit**: would a credit that scales with how many times an item has cycled through the stash outperform a single fixed value?
- **Adversarial workload validity**: counter-cap and product-score improvements may be real but latent — they could surface on a trace with hot-set drift or bursty insertion. No such workload has been introduced; all conclusions about scoring refinements are conditional on workload-a's stability.

## Per-policy notes

- **ClassicLRU (cycles 1–3 baseline)**: the unmodified baseline had slack in both eviction and stash directions simultaneously; neither frequency-aware eviction nor second-chance stash alone saturated the available gain, and they composed rather than competed.
- **LFU eviction (v001/cycle 1 winner, carried forward)**: activating the already-present `lfu_counter` was structurally minimal and produced an immediate hit-rate gain with no regressions. This has been the stable foundation for every subsequent winner.
- **LFU + second-chance stash (v001/cycle 2 winner, current baseline)**: the composed policy is now the established baseline. Its internal stash parameters (FIFO ordering, `bucket_size / 4` cap) are confirmed well-calibrated; further internal stash tuning is not recommended on this workload.
- **Promotion credit at the stash→primary boundary**: raising from +1 to +2 converted round-trips (promoted item immediately re-evicted) into genuine secondary hits. This is the only lever inside the composed policy that has shown headroom, and it has not yet been fully characterised.
