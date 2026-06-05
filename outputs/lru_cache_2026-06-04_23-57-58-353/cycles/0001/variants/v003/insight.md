# Variant v003 Insight

## Helped

- hit-rate: Scanning up to 8 LRU-end candidates and evicting the lowest-frequency one rather than the strict head kept moderately accessed entries resident longer, reducing unnecessary evictions of entries that had already proven useful at least once.

## Hurt

## Takeaway

Adding a saturating 3-bit frequency counter and a fixed-size window scan at eviction time produces a small but real hit-rate gain on this workload. The gain is modest because workload-a's access pattern gives the frequency signal enough time to differentiate hot from cold entries, but a window of only 8 limits how far back into the LRU queue the policy can reach — entries just outside the window remain vulnerable regardless of their frequency.
