# Variant v001 Insight

## Helped

- hit-rate: No movement, which is the intended composition signal. Stacking the `AtomicU64` logical-clock metadata swap (`item.rs`/`eviction_policy.rs`) and the `rotate_right`+`mem::replace` boundary-eviction put path (`associative_cache_bucket.rs`) onto the parent's already-rewritten get path leaves the final vector byte-identical, so all three cost rewrites compose with zero observable interaction.

## Hurt

- hit-rate: No regression. The logical tick preserves the exact total age order (smaller == older) so every `min_by_key`/`max_by_key` victim and promotion choice is unchanged; the put-path `self.items[..=idx].rotate_right(1)` + `replace(&mut items[0], item)` reproduces the same admission state as the old `remove(idx)` + `insert(0, item)`, touching neither the protected suffix nor `protected_count`.

## Takeaway

Consolidating the metadata (`Instant` → 8-byte `u64` tick, shrinking `Item<u64,u64>` 40→32 bytes) and put-path rewrites onto the geometry-halving parent settles the open composition question: the three independent cost rewrites stack to a byte-identical cache state and an exact hit-rate tie, so there is no hidden interaction. The actual payoff — smaller metadata footprint and the saved memmove/capacity checks — lives on size and per-op-time axes this hit-rate-only fitness vector cannot observe, making the combined profile provably free here but not provably beneficial under the current measurement.
