# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0003

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-28_11-39-56-149/cycles/0003/hypotheses/H03.md

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


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

Two variants extended cycle 0001's SLRU: a capacity-sized ghost list that sends ghost hits straight into protected (v001), and W-TinyLFU, a 1% admission-free window plus a tie-admitting count-min gate in front of SLRU (v002). A third replaced it with LIRS (v003). W-TinyLFU (v002) was the only Winner. It gained on hm_0 at both sizes and most on web_0 at the large size. Its one slight loss was web_0 at the small size.

## Patterns

- Every variant tried to reach further back for reuse evidence than SLRU's probation list can: ghost keys, a frequency sketch, or LIRS's non-resident history. What decided the outcome was how much weight each gave that evidence. Giving a returning key full protected status (v001) hurt. A relative comparison against the current victim, with ties admitted (v002), helped. This repeats cycle 0001's lesson that soft filtering beats hard filtering, with ties now on the lenient side.
- Ghost or non-resident history detects hm_0's long reuse distances well. On web_0, though, one return from outside the cache is a weak signal. It should not displace keys that were re-referenced while still resident.
- The same weak spot appeared again: web_0 at the small size. Both v002 and v003 use a fixed 1% front segment (window or HIR queue), which is too narrow there for web_0's short-range reuse. v001's ghost window, set equal to capacity, caused the opposite problem at the large size. Fixed fractions tied to capacity keep failing at one end of the size range.
- LIRS beat LRU in most cells. Its weak cell and its likely fix (a larger HIR share) match v002's, but it did not beat v002.
- For the second cycle in a row, `metadata-size` and `per-op-time` were not measured. Ghost keys, sketch counters and LIRS's three link pairs all add costs no one has checked.

## Open questions

- Does a larger or adaptive window (for example, hill-climbing the window/main split) close v002's small-size web_0 gap without giving back hm_0 gains?
- How much of the small-size slip comes from sketch tuning rather than window size: a sketch sized near capacity, frequent halving, and the double increment on miss-then-put?
- Are v002's sketch and window cheap enough on the cost axes compared with plain SLRU?

## Continuation

Build on v002 (W-TinyLFU over SLRU) and keep its tie-admitting rule. Tune the window fraction next, preferably adaptively, especially at small capacity. Separately, test a wider sketch, a longer reset period, and removing the double increment. Drop ghost-to-protected promotion. If ghost history is tried again, it should only feed the admission comparison. Put LIRS aside unless a tuned HIR share is tested directly against v002. Making the evaluator report `metadata-size` and `per-op-time` is now overdue.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
