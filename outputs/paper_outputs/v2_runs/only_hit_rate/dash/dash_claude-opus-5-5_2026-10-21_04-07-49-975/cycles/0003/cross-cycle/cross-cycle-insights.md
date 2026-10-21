## Recurring wins

- Hit-rate is decided by who gets into the scarce normal tier and who survives in the stash, not by where items sit or which free slot they use. Each cycle that produced a winner did it through admission or retention control.
- 2Q/SLRU-like split: fresh keys face a high admission bar (a stash-hit threshold, in the lineage since 0001), while proven ex-residents re-enter cheaply via pre-credit and are protected by MRU stash placement (0002). Every real gain so far came from building up this split.
- The stash being larger than the normal tier is what makes it work. It can hold unproven keys and demoted ex-residents cheaply, so filtering and recovery at the stash edge cost little hit-rate.

## Recurring failures

- Placement-only changes (MRU landing in a bucket in 0001, spilling into a free probing slot in 0001) either lose or rarely trigger. Normal buckets are almost always full together, and position is coupled to the probing-promotion bar.
- Gating the target→probing edge did nothing (0002). Front landing and pre-credit already filter it.
- Tightening the stash threshold gave diminishing returns: a small gain at 3 in 0002, and nothing on top of MRU re-entry in 0003. Don't go beyond it.
- Once MRU stash re-entry is in place, the second-order levers all came out flat (0003): stash victim choice (CLOCK-like scan), cascade routing, and stacked admission. They only reshuffle victims the stash can recover cheaply. Stacking retention mechanisms that protect the same items adds nothing.
- Cross-bucket cascade damage looked important in 0001. It stopped mattering for hit-rate once the stash recovered demoted items.

## Open questions

- Per-op-time and metadata-size have never been measured for any winner, after three cycles. This is the biggest gap. Cascade-free routing (0003 v003) is the leading per-op-time candidate because it cost no hit-rate.
- Generality: every result is from one LIRS trace at one cache size. Is the plateau a property of the design or of the trace? What happens to MRU re-entry under scans or a larger working set?
- Build integrity: a variant scored identically to its parent (0003 v001). Verify that overlays actually reach the evaluated build before treating nulls as real.
- Structural capacity, such as the stash-to-normal slot ratio or stash size, is untested and is the remaining plausible hit-rate lever.

## Per-policy notes

- Stash/normal-tier tiered design (the only policy family in the ledger so far): the current lineage is 0001 v003 (two-hit stash admission) → 0002 v003 (MRU stash re-entry for demoted items with pre-credit) → 0003 (flat; v002's CLOCK-like stash scan is the nominal winner at noise level). On this trace it looks saturated for hit-rate.
- Within this policy, the stash→target edge is the lever with leverage. The target→probing edge and placement within a bucket are low-yield.
- No other eviction policies or traces have been evaluated yet, so there are no comparative per-policy observations.
