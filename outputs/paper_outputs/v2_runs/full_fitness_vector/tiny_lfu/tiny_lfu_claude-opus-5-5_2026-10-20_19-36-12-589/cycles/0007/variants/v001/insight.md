# Variant v001 Insight

## Helped

- hit-rate: Nothing improved, but nothing was supposed to. The hypothesis required the same behaviour as the parent, and the hit rate matches exactly. That agrees with the new `word_sweep_matches_bit_sweep` test, which checks that the word-level `clock_sweep` picks the same victim and leaves the same ref bits as the old per-bit loop. So the admission duel saw the same victims as before.

## Hurt

- per-op-time: `put` p99 got worse, which is the opposite of the claim. The parent's per-bit sweep must usually stop within a few bits of the hand, because each admitted newcomer starts with a clear ref bit, so the long walks that word scanning removes were rare. The new path adds fixed work on every full-cache `put`: the `valid_mask` branch on the last word, the `!0 << pos` / `(1 << b) - 1` mask math and an out-of-line `clock_sweep` call. That extra work outweighs what the scan saves.
- per-op-time: `get` p50 and p99 also got worse even though `get` is byte-for-byte unchanged. The likely cause is layout. Adding `last_word_mask` grows `TinyLfuCache` from exactly two 64-byte cache lines to just over two, which can split the hot fields (`map`, `refbits`, `sketch`) across an extra line. Ordinary benchmark noise is also possible.
- metadata-size: The inline `TinyLfuCache` struct grew by the new `last_word_mask` field. This is a fixed cost per cache, not per entry, so the headline `Node` size did not move. The hypothesis expected this growth, but it seems to be what costs latency on both paths.

## Takeaway

Making the CLOCK sweep word-at-a-time fixed a worst case this workload almost never hits. The parent's `put` p99 tail was therefore not caused by long per-bit sweeps, and per-call overhead plus struct layout matter more than how far the sweep walks. Before trying sweep tricks again, a retry should first measure how long sweeps actually run. It should also avoid growing the struct, for example by deriving the last-word mask from `capacity` on the rare wrap instead of storing it, since a capacity of 2048 never even uses a partial last word.
