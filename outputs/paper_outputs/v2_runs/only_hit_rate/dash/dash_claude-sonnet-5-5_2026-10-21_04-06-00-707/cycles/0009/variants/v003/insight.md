# Variant v003 Insight

## Helped

- hit-rate: The one-bit reuse flag (evict a never-hit slot first) plus the second-sighting bit-array admission filter keeps one-hit-wonders out of the table, giving a small gain over baseline LRU. The gain is far smaller than the roughly 0.7 hit-rate the hypothesis aimed to retain.

## Hurt

- metadata-size: The fitness vector has no entry for this dimension, so the claimed "no added per-slot bytes" goal is unverified. The admission bit array adds a table-sized structure outside the slots, which a per-slot `Item` measurement would not capture anyway.
- per-op-time: This dimension was also not recorded, so the added admission-filter lookup and flag-clearing bucket scan on each op have no measurement. Because the hypothesis targeted this dimension, the variant cannot show it succeeded.

## Takeaway

A bit-packed reuse flag with a single-hash, no-aging admission bit array gives only a marginal hit-rate improvement over baseline. The variant's own targets, metadata size and per-op time, went unmeasured, so the compactness claim is untested. I could not inspect the overlay source in this session, so this reading rests on the hypothesis and the fitness vectors. A follow-up needs those two dimensions evaluated before it can say whether the cheap design pays off. A non-aging bit filter will also saturate over time, which may cap the hit-rate gain.
