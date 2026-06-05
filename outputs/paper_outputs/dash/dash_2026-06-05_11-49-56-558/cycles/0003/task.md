# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0003

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0003/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants probed independent axes of the Dash promotion ladder — re-entry position, eviction policy, and bucket topology — and all three improved hit-rate with no regressions. The sole Winner, v003, won by rebalancing the normal/stash split to give the hot tier equal standing with cold storage.

## Patterns

- **Topology was the binding constraint.** v003 (50/50 normal/stash rebalance) won without touching any policy logic; simply allocating more hot-tier slots was the highest-leverage move, indicating the baseline over-provisions stash for this workload.
- **Two variants diagnosed the same underlying shortage from different angles.** v001 (MRU re-entry after stash hit) shortens the time an item spends below promotion eligibility; v003 directly expands the space items compete for once eligible. These axes are orthogonal and likely additive.
- **v002's LFU gain is workload-contingent.** The benefit was explicitly tied to workload-a's frequency-skewed access pattern; LRU's recency bias was harming a stable hot set. This is a narrower win than the structural fixes and may not generalise.
- **No variant hurt anything.** The baseline has multiple independent improvement opportunities that do not trade off against each other at this capacity.

## Open questions

- Do v001 (MRU re-entry) and v003 (topology) compound when applied together, or does one subsume the other?
- Is 50/50 the optimal normal/stash ratio for workload-a, or is there a higher-gain split beyond 16/16?
- Does LFU (v002) remain advantageous on less frequency-skewed traces, or does it regress on uniform/recency-heavy workloads?

## Continuation

The hot-tier capacity bottleneck is confirmed as the dominant lever for this cycle; future hypotheses should push the normal/stash ratio further (e.g., 20/12 or 24/8) to find the gain curve's peak or breakpoint. Combining MRU re-entry (v001) with the winning topology is a natural low-risk compound to test. LFU should be held in reserve for workload-specific probing rather than treated as a general improvement.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle probed three directions branching from Cycle 0001's winning topology: compounding MRU re-entry with the 50/50 split, pushing the split further to 75/25, and introducing aging-LFU from a fresh baseline. The sole Winner, v002, confirmed that the normal/stash gain curve had not yet peaked at 16/16 — expanding to 24 normal buckets produced a further lift with no regression, and stash can tolerate reduction to at least 8 slots without penalty.

## Patterns

- **Hot-tier expansion is still the dominant lever.** v002's 75/25 split won outright, reinforcing Cycle 0001's finding that topology is more impactful than policy changes. The working set fits comfortably within 24 normal buckets, meaning capacity was still the binding constraint after the 50/50 rebalance.
- **MRU re-entry and topology are additive but unequal.** v001's compound of MRU re-entry atop the 50/50 geometry improved on v003, confirming the two axes do not interfere, but the marginal gain was modest — once the hot tier is enlarged, collapsing the re-entry penalty matters less because items already have more dwell time.
- **Aggressive per-hit aging is harmful for this workload.** v003's symmetric decay (−1 to all co-resident items on each hit) drove mildly-warm items to eviction before they could contribute hits that recency-ordered LRU would have preserved. Workload-a's access distribution punishes coarse frequency discrimination more than it rewards it.

## Open questions

- Does the normal/stash gain curve continue past 24/8, or does a further push (e.g., 28/4) hit a breakpoint where stash under-capacity starts hurting cold-item re-entry?
- Would stacking MRU re-entry (v001's insertion-position change) onto v002's 75/25 topology produce further additive gains, as it did in Cycle 0001's compound test?
- Is there a gentler aging-LFU formulation — smaller decay increment or access-count-gated decay — that avoids draining mildly-warm items on workload-a?

## Continuation

The topology gain curve is still ascending; the next hypotheses-writer should probe 28/4 and possibly the theoretical limit of maximising normal buckets to locate the peak or breakpoint. Combining MRU re-entry with v002's 75/25 split is the natural low-risk compound to run in parallel. Aging-LFU in its current aggressive form should be dropped; any return to frequency-based eviction should use a finer-grained decay to avoid the mildly-warm eviction problem observed here.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
