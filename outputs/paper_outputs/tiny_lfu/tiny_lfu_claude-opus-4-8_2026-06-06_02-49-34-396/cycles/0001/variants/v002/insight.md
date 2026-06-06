# Variant v002 Insight

## Helped

- per-op-time: Collapsing the four per-row SipHash passes into one `base_hash` plus cheap `position_for` arithmetic (shift/mul/mask) removed the dominant hashing cost from every `get`/`put`, cutting both get and put p50/p99 sharply.

## Hurt

- hit-rate: Deriving all four sketch columns from one 64-bit hash by double hashing makes the rows correlated rather than independently seeded, so collisions inflate some frequency estimates and shift a few admission decisions, nudging hit-rate down slightly.

## Takeaway

For this sketch, single-hash double-hashing is a strongly favorable trade: it roughly quarters per-op hashing work for only a marginal loss in estimate independence and hit-rate. The counter table, capacity, and LRU are untouched, so metadata-size is unchanged and the small hit-rate dip is purely the cost of reusing one hash across rows instead of four seeds.
