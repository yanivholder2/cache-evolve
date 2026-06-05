# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Shrinking the probationary victim scan from half to quarter caused the eviction loop to miss lower-frequency entries that sit beyond the LRU head quarter, occasionally evicting a higher-frequency demoted entry instead of the coldest available victim.

## Takeaway

The 1/4 scan fraction that proved sufficient for the protected segment did not transfer to probationary. Protected holds entries that have each been accessed at least once, so frequency variation is relatively compressed near the LRU end; the quarter-scan reliably finds the coldest node. Probationary mixes cold newcomers (freq=0) with recently-demoted entries whose halved counters span a wider range of positions throughout the list, meaning the minimum-frequency victim is often not within the nearest quarter. The half-list scan in v003 is genuinely load-bearing for probationary, not merely conservative.
