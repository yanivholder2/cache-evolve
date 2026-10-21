# Variant v002 Insight

## Helped

(none. metadata-size did not move: because `depth × width` is already a power-of-two multiple of 16 counters, rounding `num_blocks` added no bytes, so the blocked `Vec<Block>` has the same heap size as the flat `Vec<u32>`, as the hypothesis required. `Node` and `TinyLfuCache` are unchanged.)

## Hurt

- hit-rate: The blocked layout is not admission-equivalent to the parent's double-hashed rows, even though the hypothesis expected it to be. All four rows of a key now draw from one shared 16-counter block, using 4-bit slot fields. So any two keys that land in the same block collide on most of their counters together, not independently per row. Also, a single key's own rows hit the same slot about a third of the time, which leaves fewer distinct counters behind the min. Both effects inflate frequency estimates in correlated ways, which corrupts the `incoming_freq > victim_freq` admission test.
- per-op-time: get and put both got slower at p50 and p99, the opposite of the prediction. The likely reason is that the premise was wrong: at these capacities the parent's flat sketch is only tens to low hundreds of KB, so it already sits in L1/L2. Collapsing four row accesses into one cache line had few real misses to remove. What remains is a variable shift per row to extract each slot. There are also read-modify-write increments that can alias the same slot within a block, which may serialise the four updates through store-to-load forwarding. The parent's separate rows rarely do that.

## Takeaway

A blocked count-min sketch only pays off when the sketch is too large for cache. Here it was small enough to fit, so the layout change gave no speed gain. It also cost accuracy, because squeezing all `depth` rows into 16 slots of one line breaks the row independence that TinyLFU's min-estimate relies on. Future sketch-layout ideas should first check that the sketch's working set really exceeds L2. If they keep blocking, they should give each row its own disjoint sub-range of the block, so a key's rows can never share a counter.
