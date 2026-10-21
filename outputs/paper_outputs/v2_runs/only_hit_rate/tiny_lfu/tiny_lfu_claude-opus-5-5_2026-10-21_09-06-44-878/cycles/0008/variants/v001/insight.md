# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Halving every counter after each 10×capacity counted accesses erased the lifetime frequency that let workload-a's recurring keys beat one-off arrivals in the strict `incoming_freq > victim_freq` contest. A key whose reuse interval is longer than the reset window comes back with a halved, floor-truncated count (1→0, 3→1). It then ties with or loses to the incumbent probation victim and is rejected, so what this design loses to stale-high keys is less than what it loses by forgetting genuinely recurring ones.

## Takeaway

This was a clean, isolated test of aging. Relative to v003, the diff adds only `additions`/`sample_size`, the in-place `>>= 1` reset, and a clock that dedup keeps from double-counting. The parent had no `≤ 1` shortcuts that needed removing. Even so, hit-rate on workload-a went down. That rules out confounding from collision noise or conservative update, which the earlier anti-aging results couldn't. On this trace, the stale-high entries that lifetime counts protect are worth less than the long-reuse-distance keys that decay throws away. A 10×capacity reset period is too short for workload-a's reuse structure. Recency through count decay is not the missing lever for this SLRU+dedup design, which supports treating the current plateau as workload-a's ceiling for this family, unless a much longer reset period is tried as a last check.
