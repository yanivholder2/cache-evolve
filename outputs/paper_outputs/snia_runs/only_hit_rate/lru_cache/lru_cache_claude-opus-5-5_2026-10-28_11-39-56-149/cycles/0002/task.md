# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0002

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three independent rewrites of the baseline LRU were tried: SLRU segmentation (v001), a CLOCK second-chance ring (v002), and a TinyLFU admission gate in front of LRU (v003). The SLRU split (v001) was the only Winner. It raised hit-rate on every trace and size, and the gain grew with capacity.

## Patterns

- All three variants share one idea: a block seen only once should not displace blocks that were reused. Results depended on how firmly each enforced it. A soft filter (SLRU probation, CLOCK's unset bit) helped. A hard gate (TinyLFU's strict-greater reject) hurt the recency-driven trace.
- The two traces pull in different directions. hm_0 rewards frequency filtering at both sizes. web_0 relies on short-range recency reuse, mostly at the small size. Any mechanism that delays or blocks a block's second access, such as a short probation list or a reject on a count-of-1 tie, loses there.
- Capacity matters. With more room, SLRU's gains grew. With a large ring, CLOCK's bits stayed set and it decayed toward FIFO on web_0. At the small size, TinyLFU's web_0 loss was worst.
- No variant measured `metadata-size` or `per-op-time`; every fitness vector held only hit-rate. CLOCK's main claims (a smaller node, a cheaper `get`) and SLRU's and TinyLFU's possible costs are all still unverified.

## Open questions

- Does SLRU keep its edge on the cost axes once they are measured, and does CLOCK's expected cost advantage make up for its slight large-size web_0 loss?
- Would a larger probation share at small capacity, or an adaptive split, recover blocks whose reuse distance is longer than probation?
- Can admission filtering help web_0 if it is put behind a recency window or allowed to admit on low-count ties?

## Continuation

Build on v001's SLRU. Next, tune the probation/protected split, especially at small capacity, or try a W-TinyLFU design: a small admission-free window in front of SLRU, gated by the sketch with ties admitted at low counts. Do not try plain TinyLFU over LRU again in its v003 form. Keep CLOCK as an option for the cost axes, and test whether CLOCK-based segmentation keeps its cheap `get`. Make sure the next cycle evaluates `metadata-size` and `per-op-time`, so the trade-offs can actually be compared.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
