# Variant v003 Insight

## Helped

## Hurt

## Takeaway

On this trace, hit-rate barely moved when cascaded foreign residents were sent straight to the stash MRU end with the `PROMOTE_THRESHOLD - 1` pre-credit, instead of being pushed back into their full true-target bucket and demoting that bucket's LRU. The difference is within noise. The neighbour-cascade damage the hypothesis aimed at is therefore not a meaningful hit-rate lever in this design. This is probably because the parent's MRU-placed stash already works as a victim cache: under either routing, the item that loses its normal slot comes back cheaply on its next hit, so changing which item goes to the stash mostly swaps one recoverable victim for another. The rationale also expected lower per-op-time from fewer multi-bucket moves, but that remains untested: only hit-rate was measured, on a single LIRS trace at size 2048. (The overlay source could not be listed from this environment, so this reading assumes the implementation matches H03's described change.)
