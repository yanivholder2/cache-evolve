# Variant v002 Insight

## Helped

- hit-rate: Treating a nonzero `lfu_counter` as a protection flag makes `eviction_candidate_index` discard never-reused one-hit-wonders before touched entries, giving the ClassicLRU bucket scan resistance that the LIRS-style workload-a trace rewards.

## Hurt

- per-op-time: `eviction_candidate_index` changed from an O(1) front return to a front-to-back linear scan for the first `lfu_counter == 0`, and `get_from_position` now does an extra `last_mut` write per hit, raising both get and put latency at p50 and p99.

## Takeaway

Repurposing the already-present `lfu_counter` as a binary "reused" flag buys real scan resistance for ClassicLRU with zero metadata growth, but the linear eviction scan is the cost — the protection is only worthwhile because it turns the hit-rate gain into a favorable trade against slower evictions, not a free win.
