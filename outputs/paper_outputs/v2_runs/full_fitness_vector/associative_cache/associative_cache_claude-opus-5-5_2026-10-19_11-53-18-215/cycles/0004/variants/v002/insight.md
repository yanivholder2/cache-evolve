# Variant v002 Insight

## Helped

_(none: metadata-size did not move. The new `min_cap`, `max_cap` and `adaptive` fields live in the bucket header, which neither `Item<u64,u64>` nor `AssociativeCache<u64,u64>` measures. No per-item metadata was added, as the hypothesis promised.)_

## Hurt

- hit-rate: hit rate fell below v003 as well as below the 0.718 target, because the adaptation only really moves one way: down. Every ghost hit cuts `protected_cap` by one. On a LIRS loop/scan trace those hits are common, since keys come back just past a tiny probation window. The only growth signal is a protected hit at `pos < len0 + (items.len() - len0) / 4`, and in small buckets that quarter is empty or one slot wide. With a 4-slot pair, protected holds at most 3 items, so `3 / 4 == 0` and the cap can never grow. With 16 slots at `min_cap = 4`, only the exact protected-LRU slot counts. So buckets slide toward `min_cap`, and the cache drifts from SLRU toward plain LRU. That gives up the scan resistance that v002/v003 got from a large protected segment. The extra probation room also doesn't save the ghost keys: they are inserted at probation MRU like any cold key, which keeps them only as long as before. A bigger probation segment holds more one-shot scan keys too.
- per-op-time: get and put both measured slower at p50 and p99, more than the code change explains. `hit()` gained one branch and compare on protected hits. `pair_put` gained a short `while` loop that moves the boundary. Neither should add tens of ns. v003's insight already said its own speedup didn't match its code, so part of this gap is probably that unexplained gain going away, not a cost of the adaptive cap. Re-measure both variants side by side before blaming the change.

## Takeaway

ARC-style resizing only helps if the grow signal fires about as often as the shrink signal. Here a ghost hit (common on scan/loop traces) always shrinks protected, while growth needs a hit inside a "LRU quarter" that rounds to zero or one slot in these small buckets. The cap ratchets down to `min_cap` and wipes out the SLRU benefit. A retry needs a balanced, size-aware grow rule, for example growing on any probation-to-protected promotion that would demote a protected item, or using ARC's ghost-of-protected list as the opposing signal. It should also keep `min_cap` high enough to keep scan resistance. Weakening protected in response to probation ghosts doesn't help on workload-a.
