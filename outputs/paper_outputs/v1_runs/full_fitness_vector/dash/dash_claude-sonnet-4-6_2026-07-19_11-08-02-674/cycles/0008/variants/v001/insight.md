# Variant v001 Insight

## Helped

_(no dimensions improved)_

## Hurt

- **hit-rate**: The clock pre-pass (`clock_evict_prepare`) rotates warm items away from eviction position before the LFU-minimum scan, but on the cycle/0006/v001 base nearly every resident target item earns `CLOCK_BIT` on its first non-hottest hit — making almost all items "warm" and forcing the clock to traverse the full bucket before settling on a cold candidate that is not meaningfully worse than what the existing LFU-minimum eviction would have chosen anyway, producing a small net regression.

## Takeaway

The CLOCK_BIT bit-packing trick correctly fixes cycle/0007's corruption bug: the frequency gate now reads `lfu_counter & !CLOCK_BIT`, so the promotion threshold fires as intended. The regression reveals a deeper mismatch: because `CLOCK_BIT` is set on every non-hottest target access alongside the frequency increment, the working set in the target tier is almost uniformly warm at eviction time, leaving the clock with little discriminating power above what the parent's LFU-minimum already provides. Adding a two-chance clock on a base that already has frequency-aware eviction yields diminishing returns; future attempts should either replace the existing eviction policy with the clock or reserve the clock for tiers where frequency information is absent (e.g., the stash).
