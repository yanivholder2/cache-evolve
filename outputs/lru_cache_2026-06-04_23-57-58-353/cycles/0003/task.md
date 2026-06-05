# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003

## Winners

- v002

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0003/hypotheses/H03.md

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


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort probed three directions off the cycle-1 SLRU winner: loosening the segment-size ratio, layering frequency-aware demotion inside protected, and adding a ghost admission set. Only the ghost-admission variant (v003) won, by combining two-segment SLRU with cross-residency warmth tracking.

## Patterns

- The 80/20 protected-to-probationary split has now been stress-tested from both sides: cycle-1 confirmed it beats plain LRU, and this cycle confirmed that widening probationary to 30% hurts — the hot working set is large enough that the smaller protected segment triggers more demotion churn than the extra scan-filtering headroom saves.
- Frequency-aware demotion inside protected produced no measurable effect when the scan window covers only a small fraction of the segment and the frequency counter resets to zero on every demotion. The signal evaporates before it can accumulate durable discrimination between high- and low-reuse entries.
- Ghost admission targets a different gap than in-cache frequency: keys evicted from probationary that re-arrive before the ghost window expires have effectively earned two accesses across residency periods and deserve direct entry into protected. On a LIRS-style trace where that pattern is common, this signal is worth capturing.
- v003 was built from the baseline rather than from the cycle-1 SLRU winner, so its measured gain bundles both SLRU segmentation and ghost admission; the individual contribution of the ghost set is not yet isolated.

## Open questions

- How much of v003's gain is attributable to ghost admission alone versus SLRU segmentation? Applying ghost admission to the cycle-1 SLRU baseline (without rebuilding the segment logic) would isolate the effect.
- Could frequency-aware demotion recover if the scan window were much larger and the counter were persistent across demotion cycles rather than reset? The current failure is almost certainly a design flaw rather than a fundamental limit of the idea.
- Is ghost capacity equal to probationary capacity the right size, or would a smaller ghost window (e.g., half of probationary) suffice while reducing bookkeeping overhead?

## Continuation

Next cycle should take v003 as the new baseline and add ghost admission on top of the established cycle-1 SLRU structure as a control variant, to cleanly measure the ghost set's marginal contribution. It is also worth retrying frequency-aware demotion with a larger scan window and non-resetting counters before abandoning the idea; the cycle-2 failure was a signal-design problem, not proof that frequency is uninformative in the protected segment. Further ratio tuning of the 80/20 split is likely a diminishing return and can be deprioritized.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
