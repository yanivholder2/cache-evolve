# Variant v003 Insight

## Helped

- hit-rate: Replacing strict-LRU-head eviction with a sampled-coldest walk (up to `SAMPLE_VICTIMS = 5` oldest nodes, victim = minimum `sketch.estimate`) lets the admission filter spare hot keys that happen to sit near the head, nudging hit-rate up.

## Hurt

- per-op-time: `put` p99 regressed because each full-cache eviction now runs up to five `sketch.estimate` lookups while walking `next` from the head, instead of the parent's single head-estimate — extra tail cost on the eviction path.

## Takeaway

Widening victim selection from the single LRU head to the coldest of the five oldest entries converts the admission test from "beat the head" to "beat the weakest near-head candidate," which rescues hot keys the parent discarded — but the hit-rate lift is marginal and is bought with measurably heavier `put` tail latency from the per-eviction sample walk. Sampling helps, yet at this depth the protection it buys is small relative to its eviction-path cost.
