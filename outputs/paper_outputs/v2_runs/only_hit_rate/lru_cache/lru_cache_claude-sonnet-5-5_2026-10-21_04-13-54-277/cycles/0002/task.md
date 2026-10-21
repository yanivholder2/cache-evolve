# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0002

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three single-axis changes were tried against the baseline: a compact node layout (metadata-size), a hot-path rewrite (per-op-time), and a segmented LRU (hit-rate). Only v003, the segmented LRU, was a Winner. It was the only variant that moved a dimension the evaluator actually recorded.

## Patterns

- Each hypothesis targeted a different dimension, so the cohort did not compete on one axis.
- The fitness vectors held only hit-rate. Metadata-size and per-op-time were missing for every variant, so the v001 and v002 gains are predictions, not results.
- v001 and v002 preserved strict LRU semantics (hit-rate matched the parent). That made them neutral on the one measured axis and unable to win.
- v003 gained hit-rate by departing from strict LRU. The invariant gate accepted it. The gain came from one trace and one cache size.
- v003's added bookkeeping (a per-node flag, a second list, demotion work) probably costs memory and latency, but nothing measured it.

## Open questions

- Is the missing metadata-size and per-op-time data a harness gap or a configuration issue? It decides whether the next cycle can judge any variant.
- Does v003's hit-rate gain hold across other traces and cache sizes, or is it specific to the scan-heavy trace?
- How much metadata and latency does v003 pay for its gain?
- Do the v001 compact layout and v002 fast paths combine with segmentation without cancelling each other?

## Continuation

First, get metadata-size and per-op-time measured, or the cycle's selection signal is hit-rate alone. Then build on v003 by combining it with v001's compact links and v002's cheap promotion, so the segmented design recovers the size and speed it gives up. Do not propose more strict-LRU micro-optimizations until those dimensions are measured.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
