# Variant v001 Insight

## Helped

- (none): The second `nb /= 2; bs *= 2` step moved hit-rate the wrong way; no dimension improved over the one-halving parent.

## Hurt

- hit-rate: Looping the capacity-conserving widening step up to two times (≈4× associativity over baseline) coarsened routing one step too far — collapsing the bucket count again scattered fewer distinct keys across more crowded sets, so the wider per-bucket associativity could not recover what the lost routing precision evicted, leaving hit-rate slightly below the single-halving parent.

## Takeaway

The second halving overshoots: one capacity-conserving widening step is at or past the associativity sweet spot for this geometry, and a second step trades away routing precision for diminishing per-bucket returns. H01 is answered in the negative — the sweet spot is not wider than one halving, pinning the one-step split as optimal and the routing-loss boundary right at the second iteration.
