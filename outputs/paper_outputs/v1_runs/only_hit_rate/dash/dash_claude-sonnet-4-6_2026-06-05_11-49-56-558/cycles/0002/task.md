# Task: cycle-insight-writer

Cycle: 0002
Output dir: /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0002

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0002/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0002/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0002/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0002/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0002/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-06-05_11-49-56-558/cycles/0002/hypotheses/H03.md

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
