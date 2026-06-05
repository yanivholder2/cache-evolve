# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0004

## Winners

- v003

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0004/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0004/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0004/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0004/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0004/hypotheses/H03.md

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


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort probed three orthogonal directions off the cycle-2 baselines: isolating ghost admission's marginal contribution, rescuing frequency-aware demotion with a larger scan window and persistent counters, and testing a second-chance reprieve bit in the protected segment. Only the repaired frequency-demotion variant (v002) won, delivering a meaningful hit-rate gain by scanning the LRU quarter of protected and halving rather than resetting counters on demotion.

## Patterns

- Ghost admission is now confirmed as minimally effective on this workload. When grafted cleanly onto the cycle-1 SLRU baseline, it produced negligible lift, resolving cycle-2's ambiguity: the earlier gain attributed to ghost admission was almost entirely SLRU segmentation doing the work. The ghost window (bounded to the probationary capacity) is too narrow to intercept the reuse gap typical of this trace.
- Cycle-2's frequency-demotion failure was entirely a design flaw, not a fundamental ceiling. Scaling the scan window to a fixed fraction of the protected segment — rather than a constant small count — creates genuine selection pressure. Counter halving on demotion is the necessary complement: it lets accumulated history survive segment transitions so newly promoted nodes remain distinguishable from long-tenured ones.
- The second-chance bit failed because it required freezing node position on protected hits, which severed the continuous recency signal that makes LRU ordering useful. A one-time reprieve cannot substitute for a move-to-tail on every access; the hot entries it was meant to protect end up stranded near the demotion head anyway.
- Continuous position updates on protected hits are load-bearing. Any mechanism that suppresses them — regardless of its reprieve logic — degrades the protected segment's ordering and should not be adopted unless the position-signal loss is compensated elsewhere.

## Open questions

- Can frequency-aware demotion (v002) coexist with ghost admission without noise? Given ghost admission's negligible effect on workload-a, the combination is likely neutral, but the interaction has not been tested directly.
- Is the LRU-quarter scan fraction robust across workload shapes, or does the right window size vary with the reuse distance distribution? A workload with longer reuse distances might need a wider scan.
- Could a second-chance approach be salvaged by setting the referenced bit while still performing the move-to-tail (preserving recency ordering)? The current failure was the position-freeze trade-off, not the reprieve concept itself.

## Continuation

Next cycle should take v002 as the new baseline: two-segment SLRU with ghost admission, frequency-aware demotion scanning the LRU quarter of protected, and counter halving on demotion. Ghost admission can be deprioritized — it has been tested in isolation and found to contribute little on this workload. A productive direction is to explore whether the scan fraction (currently one quarter) should be adaptive or tuned, and whether combining frequency-aware demotion with a significantly larger ghost window (e.g., equal to total capacity rather than probationary capacity) can make cross-residency warmth visible enough to matter.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
