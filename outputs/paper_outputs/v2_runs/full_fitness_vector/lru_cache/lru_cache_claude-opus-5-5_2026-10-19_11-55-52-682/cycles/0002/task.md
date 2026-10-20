# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0002

## Winners

- v003
- v002
- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-19_11-55-52-682/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants branched off the baseline and each attacked strict LRU in its own way: SLRU segmentation (v001), a CLOCK reference-bit ring (v002) and BIP head insertion (v003). All three became Winners, and none of them got worse on any measured dimension except v003, whose metadata cost grew by one fixed per-cache word.

## Patterns

- Strict LRU recency order is a liability on the LIRS workload-a trace. Each of the three departures from it gained hit-rate: second-touch promotion, second-chance bits and loop-resistant insertion. That includes CLOCK, which was expected to lose a little.
- Per-op-time improved together with memory locality rather than with shorter code paths. v001 got smaller nodes, v002 stopped relinking on a hit, and v003 concentrated churn on a few hot slots. All of them sped up get and put at both p50 and p99.
- Node layout is the real lever for metadata-size. Removing links (CLOCK) or narrowing them to `u32` indices (SLRU) shrank the node and probably also helped speed. BIP left the layout alone and left metadata flat.
- The mechanisms are orthogonal and could stack: insertion position (BIP), a hit path with no relinking (CLOCK), and admission/protection segments (SLRU).

## Open questions

- Does the hit-rate ranking hold at other cache sizes and on traces other than workload-a? Every result so far comes from one trace at one size.
- How does CLOCK's put tail behave when most of the ring is referenced at once, forcing long sweeps?
- How much of v003's speed gain is real and how much is benchmark noise?
- Do the gains combine or overlap? All three variants target the same problem: a loop or scan flushing the reused keys.

## Continuation

Next cycle should try combinations instead of more single-axis rewrites. Good candidates are CLOCK plus a probation/protection split (CLOCK-Pro-like or a 2-bit/segmented clock), BIP-style insertion into SLRU probation, and applying the `u32`-link compaction to any policy that keeps lists. Add a check on other cache sizes or workloads and a test of the CLOCK sweep tail, so we don't overfit to workload-a at this one size. Drop further work on plain LRU with only tweaks to the order it keeps.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
