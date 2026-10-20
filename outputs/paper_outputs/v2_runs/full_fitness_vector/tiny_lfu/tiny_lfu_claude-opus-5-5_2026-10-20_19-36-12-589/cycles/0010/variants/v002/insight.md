# Variant v002 Insight

## Helped

- hit-rate: Swapping the 1-bit CLOCK reference bit for a 2-bit GCLOCK counter (0→1→3, decremented by the sweep) moved hit rate up a little. This is the first eviction-order change to beat 1-bit CLOCK, after LRU and CLOCK had tied. Residents hit twice or more now survive an extra lap, so the hand stops more often on residents hit once or never. Those are the victims the unchanged `incoming_freq > victim_freq` duel is meant to defend. The gain is small, though: with a 2-bit counter only "hit once" and "hit twice or more" can be told apart, so a resident with many hits gets the same protection as one with two.
- metadata-size: Unchanged, as intended. `Node<u64,u64>` stays at 16 bytes because the counter lives in a second bit-plane next to `refbits`, not in the node. The `TinyLfuCache` header stays inside two cache lines because the implementation changed both planes from `Vec<u64>` to `Box<[u64]>` and narrowed `capacity` to `u32`. That offset the added field, which the hypothesis's "one extra `Vec` header" would not have done.

## Hurt

- per-op-time: The p99 tails got worse on both ops, and the `put` p50 rose slightly. On `put`, the full-miss sweep now loads and stores two words per visited slot instead of one. A resident with u=3 also needs two visits before it reaches 0, and the debug bound grows to three laps, so long sweeps get longer when many residents are hot. On `get`, `set_ref` now does an extra dependent read-modify-write on the `refbits_hi` word. The `get` p50 did not move, which fits the parent's finding that the median `get` is limited by the `HashMap` lookup and hashing, not by ring metadata.

## Takeaway

Changing eviction order inside the main region gives a small hit-rate gain where admission-side changes had given nothing. Keeping the admission duel fixed while giving residents with repeat hits more chances to survive does change which cold resident the duel defends. The cost is tail latency from a second bit-plane on both the hit path and the sweep. A next step could interleave the two bits per slot in one word, which would remove the extra RMW while keeping GCLOCK's victim choice. Another option is a wider (3-bit) counter, which would show whether more levels add hit rate beyond what this first one bought.
