## Recurring wins

- Changes with a clear mechanism are the ones to credit. The scan consolidation (Cycle 3) removed redundant linear scans and left hit rate and metadata unchanged.
- Preserving the small queue's one-hit-wonder filtering works. A separate bounded main-eviction ghost (Cycle 2) gave a modest hit-rate gain. It succeeded where recycling into the shared ghost (Cycle 1) failed.
- Constant-only small-queue ratio changes are cheap. The gain has flattened: 10% helped modestly and 5% only marginally. The knob is near exhaustion.

## Recurring failures

- Latency is not a trustworthy selection signal. In every cycle, variants with no hot-path mechanism showed latency shifts, so regressions in hit rate were promoted by noise (ghost recycling, freq>=1 promotion, the 15% ratio, the rotation cap).
- Letting more one-hit-wonders into main, or taking capacity from main, lowers hit rate. Examples: looser promotion, a larger small queue, and recycling main evictions into the shared ghost.
- Guards on already-bounded behavior (the rotation cap, given a saturating frequency counter) cost hit rate and buy nothing.
- Infrastructure failures (a Maven 429) can hide ideas. Do not treat the lost variant as evidence against its idea.
- Every cycle has used one trace (workload-a) at one cache size, so no conclusion is shown to generalize.

## Open questions

- Is the separate main-ghost gain larger than run-to-run noise? How should the ghost be sized, and does it need a freq head start on re-entry? Is its metadata and put cost worth it?
- Does the 5% small-queue ratio hold on other traces and cache sizes, or is the optimum workload-dependent?
- Can an index such as a hash map replace the linear membership scans that dominate `put` cost?
- Can the tests deleted by the scan refactor be restored without losing its gains?
- Can repeated runs or multiple traces separate real effects from noise in the fitness signal?

## Per-policy notes

- S3-FIFO (small/main/ghost):
  - The baseline promotion threshold is retained. The freq>=1 variant lost.
  - The small ratio is around 5-10%. Larger ratios lose.
  - The shared demotion ghost must stay clean. Main evictions belong in a separate ghost.
  - The frequency counter saturates low, so rotation is already bounded.
  - The next steps are scan and index work for `put` cost, and tuning the separate ghost. Judge by hit rate across traces.
