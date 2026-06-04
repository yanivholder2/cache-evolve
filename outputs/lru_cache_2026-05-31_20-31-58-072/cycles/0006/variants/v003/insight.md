# Variant v003 Insight

## Helped

- (no fitness dimension moved in this variant's favor; both prescribed fixes combined regressed against the symmetric-S4LRU parent)

## Hurt

- hit_rate: the top-heavy split shrinks L0 from `capacity/4` (128) to `86`, so brand-new keys are evicted from L0's tail even sooner than in the parent — the climbing ladder's bottom rung now has *less* room to let a warm key earn its first promotion, which is the very bottleneck cycle-5's insight identified.
- hit_rate: the L3 ghost ring only fires on the direct-admit path (a key evicted *from L3* and then re-referenced); with L0/L1/L2 each compressed to ~85 slots, far fewer keys survive the three-hop climb to L3 in the first place, so the ghost ring's shortcut almost never activates and L3's doubled retention room sits underused.
- hit_rate: turning L3 into half the cache while keeping the strict three-hop promotion rule means a key must still be hit three times before it can benefit from the protected tier — the asymmetric split makes the prize bigger without making the entry condition easier, so the net effect is that the working set spends more time fighting for room in the now-smaller L0/L1/L2 staging area.

## Takeaway

Applying the cycle-5 two-fix prescription verbatim — top-heavy split plus L3 ghost ring — *underperforms* the symmetric S4LRU parent on lirs:multi1 at 512, decisively answering the cross-cycle open question in the negative: the segmented-LRU skeleton cannot match the CLOCK-Pro-lite plateau on this trace by structural patching alone. The two fixes interact badly: enlarging L3 starves the staging levels that gate promotion into L3, and the ghost-ring shortcut depends on keys having once reached L3, which the starved staging levels now prevent. Future variants should retire the multi-hop promotion ladder rather than try to redistribute room around it.
