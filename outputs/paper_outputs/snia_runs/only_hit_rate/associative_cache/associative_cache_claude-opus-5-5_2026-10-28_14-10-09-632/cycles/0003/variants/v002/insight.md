# Variant v002 Insight

## Helped

- hit-rate: hm_0 at 16384 was the only cell that improved, and the gain was small. On this trace a key that misses and is still in the per-bucket ghost ring has real reuse. Admitting it with `lfu_counter = 1` lets it survive one extra sweep over the cold items that come in at 0. That is the mechanism the hypothesis predicted, but it only shows up in the small, high-reuse cell.

## Hurt

- hit-rate: web_0 dipped slightly at both 16384 and 131072. On a trace with weak reuse, a ghost hit is often a coincidence and not a sign of a hot block. Each warm admission makes the GCLOCK hand step past one more item, and the sweep then evicts a younger cold item that might still have been reused. The net effect is a small amount of pollution, not scan protection.
- hit-rate: hm_0 at 131072 did not move, and the summary is flat to marginally down overall. So the extra ghost state, plus the linear `ghost_take` scan on every miss, bought no aggregate benefit.

## Takeaway

The ghost ring holds `bucket_size / 2` keys, so it only remembers about half a rotation of evictions. A block whose reuse distance is longer than one hand rotation, which is exactly the block the hypothesis was aimed at, has usually already been overwritten in the ring by the time it comes back. A credit of 1 is also worth only a single extra sweep. As built, the change only catches short-distance re-references, and GCLOCK's counters already handle those. If ghost admission is worth pursuing, the ring has to cover at least one full rotation (`GHOST_DIV <= 1`) or the warm credit has to be larger. Otherwise this direction should be dropped instead of tuned further. The per-miss linear ghost scan also adds unmeasured cost on the miss path.
