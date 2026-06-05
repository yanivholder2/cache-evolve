# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0005

## Winners

- v002

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0005/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0005/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0005/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0005/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-04_23-57-58-353/cycles/0005/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

All three variants extended the cycle-3 frequency-demotion baseline in different directions — scan-fraction widening, ghost-window expansion, and frequency-aware eviction in probationary. Only the probationary extension (v003) won, by ensuring that the freq counter preserved across demotion actively shields recently-warm entries at both eviction sites.

## Patterns

- The frequency counter and counter-halving-on-demotion design have been the load-bearing mechanism since cycle 3, but it was applied asymmetrically: protected-segment demotion respected frequency, while probationary eviction discarded it via blind LRU. v003 closed that gap — the signal survived the segment boundary but was being thrown away at the next eviction decision.
- Scan-fraction tuning above 1/4 in the protected segment is now confirmed to offer negligible returns on this workload. Going from 1/4 to 1/3 moved hit rate only marginally, indicating the frequency variation near the LRU end of protected is mostly captured by the narrower window already.
- Ghost-window expansion to full capacity regressed hit rate: reuse events at distances between probationary capacity and total capacity on this trace are not reliable locality signals, and fast-pathing those keys into protected pollutes it. The effective ceiling for the ghost window sits well below total capacity, and the ghost-window expansion thread is now largely exhausted.

## Open questions

- Is the probationary LRU-half scan fraction robust, or should it be adaptive? A workload with longer reuse distances or a more skewed access distribution might need a different fraction at the probationary eviction site.
- With frequency awareness now applied at both eviction sites, is there any remaining structural gap, or have the major policy improvements been captured? A direct comparison against a wider workload suite would clarify how general these gains are.
- Could an intermediate ghost window (e.g., 50% of total capacity, between probationary capacity and the failed full expansion) recover some cross-residency signal without the admission-quality problem seen at full capacity?

## Continuation

Next cycle should take v003 as the new baseline: two-segment SLRU, counter-halving demotion, frequency-aware eviction scanning the LRU quarter of protected and the LRU half of probationary. Ghost-window expansion can be deprioritized — the full-ceiling test failed and intermediate values are unlikely to move the needle meaningfully on this trace. The most productive direction is probing the probationary scan fraction (is 1/2 near optimal, or does a smaller fraction suffice?), and testing whether either or both scan fractions should vary proportionally with observed reuse-distance patterns rather than being fixed.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
