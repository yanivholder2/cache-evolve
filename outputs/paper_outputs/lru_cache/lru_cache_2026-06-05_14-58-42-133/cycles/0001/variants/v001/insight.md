# Variant v001 Insight

## Helped

- hit-rate: Splitting the single LRU list into probationary/protected segments and promoting only on a second touch keeps one-hit-wonders confined to the probationary segment, so they are evicted first instead of displacing repeatedly-reused keys — exactly the scan-resistance the hypothesis targeted.

## Hurt

- (none): hit-rate is the only measured dimension and it moved up; no tracked dimension regressed.

## Takeaway

For workload-a, gating entry into the protected segment behind a second hit (and always evicting the probationary-LRU first) is enough to meaningfully resist pollution — the SLRU promotion rule, not the ~80% protected cap alone, is what earns the gain over a flat LRU.
