# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0002

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-10-21_09-06-52-796/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three variants branched from the baseline:
- **v001:** an admission-filtering policy (S3-FIFO).
- **v002:** a cheaper LRU approximation (CLOCK).
- **v003:** a representation-only control that keeps strict LRU and adds Fx hashing and `u32` links.

v001 won by raising hit rate on the scan- and loop-heavy workload-a. v002 roughly matched LRU's hit rate, and v003 left it unchanged, as designed.

## Patterns

- **Only hit rate was measured.** None of the variants reported `metadata-size` or `per-op-time`. Two of the three hypotheses (H02, H03) were aimed mainly at those dimensions, so their main claims are still untested.
- **Admission filtering is what helped.** On this trace, the gain came from changing which keys get admitted. Changing how recency is tracked did not help.
- **Exact LRU ordering adds nothing to hit rate here.** CLOCK's coarser second-chance recency held hit rate at LRU's level or slightly above. Precise LRU order can be given up without a hit-rate penalty.
- **All three variants remove or shrink the `prev`/`next` links.** Each predicts a smaller node and cheaper hits, but none has been measured. v001 also adds ghost structures stored outside `Node`, which could cancel its memory savings.

## Open questions

- Does v001's ghost queue and set add more per-key memory than dropping the links saves? And is its `get` actually faster?
- Does CLOCK's victim sweep cause tail-latency spikes on `put` when most entries are marked referenced?
- Are v003's representation changes (Fx hasher, `u32` links, MRU fast path) independent of policy? If so, they could be stacked onto v001 or v002 for a speed gain.

## Continuation

Re-measure `metadata-size` and `per-op-time` for v001, v002 and v003 before building on them. If the harness can't produce these dimensions, fix that first. Then use v001 as the parent and try:
- shrinking or compacting its ghost structures,
- grafting v003's hasher and narrow indices onto it,
- tuning the size of the small probation queue.

Don't spend further cycles on CLOCK-style recency tweaks unless measurement shows a clear memory or latency win.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
