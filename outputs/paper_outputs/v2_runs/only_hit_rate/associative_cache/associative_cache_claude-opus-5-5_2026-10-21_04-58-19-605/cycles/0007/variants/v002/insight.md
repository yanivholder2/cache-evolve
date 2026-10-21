# Variant v002 Insight

## Helped

(None. hit-rate was the only dimension recorded, and it did not improve.)

## Hurt

- hit-rate: In a full 16-slot bucket, a ghost-hit key now goes in at index 5 instead of index 3. That gives it two more misses before it is evicted. It also means the keys at indices 3 and 4, which are newcomers and aged hit keys, are now evicted before it. On workload-a those two slots were worth slightly more holding their current keys than giving a returner extra grace. The drop is very small and the hypothesis's 0.723 threshold was not reached, so H02 is refuted. The size of the drop suggests the hit-rate curve is flat between index 3 and index 5, not that hit rate falls off sharply.

## Takeaway

The ghost-hit insert depth on the 1/8 LRU+ghost line has reached its peak, or passed it slightly, at index 3 (1/4). Moving it to index 5 (3/8) gives nothing back. One likely explanation is that keys returning from the 4-entry ring mostly come back within about three misses of being evicted. If so, extra depth only protects returners that won't be reused in time anyway. Insert depth alone cannot close the LRU line's gap to CLOCK+ghost, so the remaining levers are the ghost signal itself (ring length, or what gets recorded) and not where a ghost hit lands. The added `idx.min(self.items.len())` guard has no effect, because 3/8 of len can never be more than len. The change is only the constant. It was evaluated on one trace at one size, and a difference this small may be within run-to-run noise.
