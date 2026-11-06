# Variant v002 Insight

## Helped

(None. No dimension moved by more than noise in the hypothesis's favour.)

## Hurt

- hit-rate (hm_0, size 131072): This was the target cell, and it did not move (a change at noise level, slightly down). With `Ghost::new((normal_cap + stash_cap) * GHOST_CAPACITY_MULTIPLIER)`, the ghost now remembers stash drops across gaps of about two caches' worth of evictions, and still nothing was recovered. So the lost reuse at this size is not just past 1× ghost reach. The reach explanation from the rationale is refuted. The remaining causes lie elsewhere, for example how much a single `lfu_counter = 1` step can protect a key in `lru_tail_lfu_victim` once halve-on-cap ageing runs, or keys that are never ghosted because they leave through paths other than stash drops.
- hit-rate (hm_0, size 16384): This is the only cell that clearly moved, and it went down. At the small size the 1× table already covered the useful reuse gaps that the parent captured. Doubling the table lets older, colder returners also win the `lfu_counter = 1` admission bonus. They then compete in the 8-slot tail scan with keys whose reuse is nearer, which dilutes the bonus. Fewer collision overwrites mean more stale fingerprints survive to fire.
- hit-rate (web_0): Flat at both sizes, as the rationale predicted. Scan keys in this trace rarely come back, so a larger table seldom fires. The 16-bit fingerprint keeps false matches negligible even with twice the slots.
- (Not measured: metadata-size, which rises from about 2 to about 4 bytes per resident slot, and per-op-time, where the lookup is still one splitmix64 mix plus one compare but locality in the table is worse. Neither appears in the fitness vector.)

## Takeaway

Making ghost reach longer than one cache does nothing for hm_0 at 131072, so reach is ruled out as the limit there. It also hurts hm_0 at 16384 slightly, because more distant returners now share a bonus that was worth more when only near returners got it. Ghost size should go back to the parent's 1× value. The next experiment on the large-size hm_0 cell should change how ghost hits are protected (the strength or durability of the admission bonus against ageing and tail-scan eviction), not how far back the ghost remembers.
