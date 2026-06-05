# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Halving the probe zone from one-half to one-quarter of primary capacity shortened the filter window so much that items needing only a second access to earn promotion were FIFO-evicted from probe before their confirming hit arrived, eliminating hits that the larger probe had previously captured.

## Takeaway

The probe zone size is not freely reducible: at half-capacity the zone was already calibrated to the inter-access gap of re-used items on this Zipfian workload, and cutting it to a quarter forces premature eviction of items that would have been promoted. The gain from probe admission control is not monotone in probe size — there is a minimum window below which confirmed-reuse items are lost before they can be promoted, and the half-capacity ratio is closer to that floor than to a generous surplus.
