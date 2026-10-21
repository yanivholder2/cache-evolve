# Variant v003 Insight

## Helped

- per-op-time (put): Put misses now fill an empty slot or overwrite the tail victim, then do one O(1) swap to the len/4 slot. This replaces shifting elements through the bucket, so put latency at both median and tail is lower.

## Hurt

- hit-rate: Fixed len/4 insertion with in-place victim overwrite did worse than baseline on workload-a. The new entry no longer lands at the baseline's position, and the swap puts whichever entry sat at len/4 into the tail slot, where it is evicted next. Placement alone does not improve hit rate and here it costs some.
- per-op-time (get): Get latency is slightly worse. Get and promotion code is unchanged, so this is probably noise or a side effect of the changed bucket layout, not a real regression.

## Takeaway

Swap-based fixed-position insertion makes put cheaper, but it does not account for any hit-rate gain from adaptive insertion. As a control, it shows that placement without adaptivity loses hit rate, so any gain from a similar change must come from the adaptive part. It also suggests the swap evicts a useful entry from the len/4 slot.
