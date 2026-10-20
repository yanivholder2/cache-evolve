## Recurring wins

- Segmentation (segmented LRU) is the established source of scan-resistance hit-rate; no need to re-prove it.
- Layout/mechanics changes (u32 links, lighter hasher, head-touch early return, move-based eviction) gave speed/size on the plain LRU without touching hit-rate. Hit-rate comes from policy, speed/size from mechanics; the axes are independent.
- Removing the hash avalanche fold on the segmented variant was a real put-speed and size gain, with no hit-rate loss.
- Ghost history used as a scan filter on insertion position (cold keys go to the next-to-evict slot unless remembered) is the one structural hit-rate gain since segmentation. It cost put latency, especially the tail.
- Ghost overhead trimming (smaller table, fewer/single hashes, gated lookup) cut the put tail while hit-rate held.
- Protected-share increases gave small hit-rate gains, but each step gained less.
- A capped second-chance eviction walk cut get and put latency, including put p99, at a hit-rate cost.

## Recurring failures

- Shifting work between operations (lazy demotion, reference-bit get path) only moves the promotion/demotion relink cost. It does not remove it.
- Skipping relinks only near the head did not pay off: too few hits land there, so the added branch cost more than it saved.
- Ghost-hit promotion (to protected, or to probation MRU) hurt hit-rate. The filter works by keeping suspicious keys cheap to evict, not by rewarding remembered ones.
- Micro-optimizations (key clone removal, node-slot reuse, further hasher tweaks, ghost shrinking beyond one halving) are invisible. Map remove/insert dominates put.
- Demotion-position tweaks and `put`-update semantics changes were neutral or slightly negative.
- Bundling several changes made attribution impossible. Authors could not read diffs, so attribution is inferred.
- "Winner" status since about cycle 0005 often reflects noise tolerance, not improvement. Small per-op-time deltas appear even on untouched paths and are unattributable.

## Open questions

- What is the per-op-time noise floor? A repeat-run measure is still missing, and it blocks crediting any latency claim.
- Does the ghost-gated segmented design hold on non-scan workloads? This check has been pending for many cycles.
- Can map operations per put be cut structurally (entry-style combined lookup/insert, cheaper map) to move the median put?
- Is the remaining hit-rate gain in admission policy or cache sizing, since insertion position and segment share look exhausted?
- Heap-allocated ghost memory is invisible to the per-node size metric. A heap-inclusive memory measure is needed.
- Can the eviction-walk cap become adaptive, keeping the put p99 gain without the hit-rate loss?
- Does u32 links give a speed win on a segmented policy? Only size has been shown.

## Per-policy notes

- Plain LRU: u32 links and a lighter hasher were pure wins in cycle 0002. This is saturated.
- Segmented LRU: the cost is promotion/demotion relinking. Fold-free hash plus reference bit is the baseline. The protected share and demotion position are exhausted. The cap on the second-chance walk is a latency/hit-rate dial.
- Segmented LRU with ghost-gated insertion (the cycle-0008 baseline, smaller ghost table, single-hash gated eviction path) is the current parent. The ghost table has headroom, but shrinking it further buys nothing measurable. Next levers are structural: fewer map ops per put, or an admission policy.
