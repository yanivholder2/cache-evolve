# Variant v002 Insight

## Helped

- per-op-time: put p99 fell sharply from the parent's bad tail. Sizing the index to exactly capacity (no slack) and adding the avalanche finalizer in `finish()` removed the cold-insert outliers. This supports the parent's guess that the oversized sparse map and weak mixing caused the tail.

## Hurt

- per-op-time: get p50 and p99 both regressed from the parent's gains, so the hasher's gain was only partly kept. The extra xor-shift/multiply/xor-shift finalizer adds a fixed cost to every lookup, which is the dominant cost of a get. A table with no slack is also fuller, so probe sequences are longer.
- per-op-time: put p50 regressed too. Each put pays for the finalizer on its lookup and insert, and a tightly sized table probes more on every insert, so the median rose even though the tail shrank.
- per-op-time: the headline summary is the get p50, so it got worse overall even though the put tail improved.

## Neutral

- hit-rate and metadata-size did not move. The change touched only hashing and pre-sizing, and no policy logic or entry layout.

## Takeaway

The two changes fixed the put tail but gave back much of the get speedup. Better mixing and tighter sizing trade median latency for tail latency, and the hash finalizer is not free on a path that is almost all hashing. The parent's cheap Fx hash with a modest amount of slack, or a lighter finalizer, would probably keep more of the get gain. This variant has no fitness result that separates the two effects, so the split between finalizer cost and probe length is a guess.
