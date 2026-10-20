# Variant v001 Insight

## Helped

- hit-rate: Halving the direct-mapped ghost (`gsize = next_power_of_two(capacity) / 2`) on top of v002's main-eviction ghosting clears the hypothesis's "strictly above v002" bar, but only by a sliver. The two cycle-0008 gains did not add up. The combined gain is far smaller than the half-ghost's own gain on v001's layout. A shorter ghost memory still helps a little here: fingerprints expire sooner, so fewer stale or aliased entries send keys to the main tail by mistake. But half the effect is gone, because small-head and SIEVE-main victims now compete for half as many slots and push useful small-queue fingerprints out sooner.
- per-op-time: The small `put` p99 improvement fits the ghost probe and stores touching a 4 KB table instead of an 8 KB one. The change is too small to tell apart from noise. `put` p50 and the untouched `get` path did not move.

## Hurt

(none) No dimension got worse. `metadata-size` is unchanged because `Node` is still 16 bytes and the `LruCache` struct holds only the `Vec` header. The ghost's heap allocation does halve, but this metric does not count it.

## Takeaway

On workload-a, the two ghost tweaks interfere rather than stack. Shrinking the ghost and feeding it SIEVE main victims both target the same thing: a cleaner, more selective ghost. Once main victims share the table, halving it mostly adds collision dilution and very little extra selectivity. The combination is still a free, non-negative change and worth keeping as the new base. But the leftover gain is so small that it confirms the hypothesis's own exit condition in spirit: tuning the direct-mapped ghost's size and inputs is exhausted on this trace. Further hit-rate gains will need a different policy lever.
