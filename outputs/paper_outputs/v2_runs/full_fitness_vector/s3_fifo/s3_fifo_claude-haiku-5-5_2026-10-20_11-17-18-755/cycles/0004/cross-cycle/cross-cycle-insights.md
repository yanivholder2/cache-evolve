## Recurring wins

- Replacing stored ghost keys with fingerprints moved put-path cost in the same direction in two separate cycles (0003, 0004), with hit rate unchanged. It is the only edit with a repeated, attributable benefit so far.
- Policy-preserving data-structure changes (hash membership index, sequence-tagged ghost) left hit rate exactly unchanged, which makes them safe to evaluate on cost alone.
- Edits confined to the put path have been the ones with a mechanism that matches the measured effect; get-path changes have not.

## Recurring failures

- Hit-rate losses from changing how much the ghost or small queue admits: lowering the small-queue promotion threshold (0001) and enlarging the ghost queue toward full capacity (0004) both moved singly-reused keys into main without a matching reuse gain.
- Stacking a lowest-frequency victim choice on the enlarged small queue (0003) lost hit rate, and the rotation budget it depends on likely runs out less often once the small queue is larger.
- Sequence-tagged ghost bookkeeping (0002) added per-eviction put cost without any policy gain.
- Moving the frequency counter into a parallel side array (0004) grew the container enough that total footprint got worse, and it added an indexed read on every hit.
- Frequency aging on main-queue rotation (0003) showed no effect on the one trace reported; it has never been shown to change an eviction.
- Several Winner labels rest on hypothesis text and fitness deltas rather than verified mechanisms, and on single traces, so they are weaker than the labels imply.

## Open questions

- Does the fingerprint ghost's put-latency gain hold across other traces and capacities, and does its linear ghost-hit scan or added container metadata show up in tail latency at larger sizes?
- Is the get-path latency shift real? It has appeared in variants with no get-path edit (0001–0003) and in one variant with a plausible code-level cause (0004). A baseline rerun with no code change has not been done.
- Is there a ghost capacity between main-sized and full capacity that keeps some reuse benefit without the hit-rate loss?
- Does the small-queue share increase hold on other traces and capacities, or is the gain specific to the workload tested?
- Where is the dominant put miss-path cost, if the ghost `position` scan is not it?
- How often does the rotation budget actually run out under the evaluated workloads, and does the lowest-frequency victim choice help when it does?

## Per-policy notes

- Small queue: a larger share gave a small hit-rate gain on one trace (0002); promotion-threshold changes (`freq > 0`) cost hit rate and gave no attributable latency benefit (0001). Keep the share question open; drop threshold tuning.
- Main queue: the second-chance rotation budget is a bounding change that trades forced evictions of hot entries for put cost (0001). Post-budget victim choice helped only in the one case tested and lost hit rate once stacked on the larger small queue (0002, 0003). Frequency aging is untested on a trace where the rotation pass runs long (0003).
- Ghost queue: the fingerprint representation is the clearest put-path win (0003, 0004), but it adds container overhead and linear ghost-hit cost. Sequence tagging added put cost without policy gain (0002). Enlarging toward full capacity lowered hit rate (0004). Hash membership is policy-neutral (0001).
- Per-entry metadata: moving the frequency counter to a side array did not reduce total footprint and touched the get path (0004). No variant has yet shown a footprint gain that holds at the structure level.
