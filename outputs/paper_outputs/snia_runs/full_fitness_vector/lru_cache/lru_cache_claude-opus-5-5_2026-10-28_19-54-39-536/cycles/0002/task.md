# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0002

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_19-54-39-536/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants branched from baseline, each aimed at a different axis: SLRU for hit rate (v001), CLOCK lazy promotion for get latency (v002), and `u32` link compaction for metadata size (v003). Both Winners (v002, v003) shrank node metadata. v001 improved hit rate, but paid for it with larger nodes and slower operations.

## Patterns

- Shrinking metadata was the most reliable win. Two different routes got there: dropping the links entirely (v002) and narrowing them (v003). The node layout has no free padding, so any added field costs a full word unless it reuses space the node already has. v001's prediction that padding would absorb its new byte was wrong.
- get latency hardly moves whatever the list does. Removing relinking (v002) and compacting nodes (v003) both left get p50 roughly flat. This points to the `HashMap` probe, not list maintenance, as the dominant cost of a hit.
- Frequency or scan filtering gives modest hit-rate gains on the MSR block traces. The gains depend on cache size. SLRU helped mostly at the large size, while CLOCK's single bit helped hm_0 at both sizes and web_0 at the small size, and lost slightly on web_0 at the large size. Neither approach dominated.
- Policy changes moved several axes at once, with mixed results. Representation-only changes (v003) moved one axis cleanly and left hit rate exactly unchanged.

## Open questions

- Can SLRU's frequency filter run on a compact node (a segment flag in a link high bit or sentinel, on top of `u32` indices) without the metadata and latency penalty?
- Would an adaptive or larger probation share fix SLRU's weakness at the small cache size?
- Can CLOCK's put tail latency be bounded, for example with a capped sweep or amortised hand advance, without losing its median gain?
- Would a faster hasher or a different map layout finally move get p50?

## Continuation

Treat v003's `u32`-index layout as the default base for later variants. It is policy-neutral and costs nothing. Combine it with scan resistance: try SLRU with an in-node flag and/or a tuned probation share, or CLOCK with a bounded sweep. To lower get latency, target the hash map. Drop further list or node-compaction ideas as a speed lever.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
