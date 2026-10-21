# Variant v003 Insight

## Helped

- hit-rate: no movement versus the parent; the frequency scan never changes which victim is picked, so there is no gain.

## Hurt

- hit-rate: no regression either, but the extra work in `put` (a 4-node walk on each eviction) and the added `freq` byte per node buy nothing.

## Takeaway

The `freq` counter is dead signal. Every hit moves the node out of probation, and demoted nodes are reset to 0, so every probation node always has freq 0. The min-freq scan therefore always ties and falls back to the oldest entry, which is exactly the parent's probation-LRU eviction. A frequency signal only helps if it survives in the segment being scanned. Here it would have to persist across demotion (not be zeroed), or the scan would have to cover protected entries.
