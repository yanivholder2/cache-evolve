# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002

## Winners

- v003

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0002/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants tested reuse-aware eviction policies against a scan-heavy workload; the two-segment SLRU design (v001) was the sole Winner, producing the largest hit-rate gain by explicitly partitioning entries that have proven their usefulness from those that have only been seen once.

## Patterns

- Every variant improved hit rate over the baseline — none regressed — confirming that stock LRU's scan-pollution vulnerability is real and addressable by multiple independent mechanisms.
- The cohort converged on a single root cause: baseline LRU does not distinguish one-hit entries from reused ones, so cold-traffic bursts displace warm entries. All three strategies attacked exactly that axis, just at different points in the eviction pipeline.
- SLRU (v001) won decisively because it acts at insertion time, routing every new entry into a probationary buffer before it can touch the protected working set. CLOCK (v002) and the frequency-window scan (v003) act only at eviction time, which is a later and weaker signal.
- The frequency-window gain (v003) was explicitly bounded by the fixed 8-slot look-ahead; entries just outside the window remained vulnerable regardless of their access count, leaving a structural ceiling on improvement.

## Open questions

- Can CLOCK or frequency-window eviction be layered *inside* the SLRU protected segment to add a second line of defense once entries are promoted?
- Would a larger or adaptive frequency-window (beyond 8 candidates) close the gap with SLRU on this workload, or does the eviction-time limitation make it a dead end regardless of window size?
- The CLOCK variant eliminates linked-list pointer maintenance entirely — does that structural simplification yield measurable throughput benefits under high concurrency or very large capacities?

## Continuation

Next cycle should advance SLRU (v001) as the new baseline and probe the 80/20 protected-to-probationary split ratio — a tighter or looser split may further tune the balance between scan resilience and working-set capacity. It is also worth exploring whether layering frequency-based demotion on top of SLRU (evicting low-frequency entries from the protected segment back to probationary rather than strict LRU demotion) closes any remaining gap; the frequency-window idea is not exhausted, just constrained by its current eviction-only placement.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
