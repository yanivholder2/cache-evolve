# Variant v002 Insight

## Helped

- hit-rate: The value-less A1out ghost queue lets a key that was evicted from A1in be readmitted straight to the hot Am LRU on its next put, so reuse-distance-heavy accesses survive instead of repeatedly cycling through the cold FIFO.
- hit-rate: Splitting the arena into a small A1in FIFO (kin = capacity/4) and an Am LRU filters one-hit scan keys out before they can pollute the proven-key segment, protecting frequently-reused entries from eviction.

## Hurt

(No other fitness dimension is measured for this subject, so nothing else moved.)

## Takeaway

Replacing the single LRU list with a 2Q policy — A1in FIFO + Am LRU + a ghost-only A1out — pays off on this trace: the ghost queue captures returning keys without spending an arena slot, and the FIFO admission filter keeps scans from displacing proven keys. The win comes specifically from separating "seen once" from "proven" residency, not from any change to the O(1) arena reuse the baseline already had.
