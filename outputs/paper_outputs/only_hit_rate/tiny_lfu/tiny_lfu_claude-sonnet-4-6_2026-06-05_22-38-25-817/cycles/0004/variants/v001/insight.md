# Variant v001 Insight

## Helped

## Hurt

- hit-rate: The ghost ring-buffer fast-tracks re-admitted items into protected, but those items are precisely the ones that *lost* the TinyLFU admission contest (they were evicted because their frequency was lower than the incoming key's), so promoting them to protected on re-insertion displaces legitimately hot residents and increases churn in the 90% protected segment.

## Takeaway

Adding a capacity/4 ghost buffer on top of v003's promotion gate did not improve hit-rate on this Zipfian trace — it slightly worsened it. The flaw is that the ghost buffer records victims of the frequency-gated eviction path: these items were already identified as less-hot than their replacements, so fast-tracking them into protected upon return misidentifies a low-frequency survivor as a high-priority item, cascading unnecessary demotions out of a segment that v003's `PROMO_THRESHOLD` had already stabilized. On a trace with a clean Zipfian hot set, the `was_protected` bypass handles the legitimate "evicted-by-overflow" case adequately, leaving little room for a re-admission ghost mechanism to add value.
