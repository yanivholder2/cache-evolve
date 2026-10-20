# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0006

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0006/hypotheses/H03.md

## Prior cycles digest

### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort tried three changes around the one-bit "seen in stash" marker: stacking the second-hit promotion gate on the MRU stash-hit landing, using the marker to prefer never-re-hit stash entries as eviction victims, and moving the marker into a spare tag or bitmap bit. Only v001 won, and its gain is a small hit-rate improvement on a single trace.

## Patterns

- Stacking the two stash-hit levers looks complementary. The second-hit gate added on top of MRU landing gave a small gain, but it has been measured on only one workload and one capacity.
- Using the marker as an eviction signal hurt. Preferring never-re-hit stash entries as victims discarded recently arrived items that had not yet had a chance to prove reuse.
- The marker-relocation variant reports no hit-rate change. Its evidence does not show whether the marker moved into a spare bit at all, or whether the variant fell back to no change, so its cost is unknown.
- Metadata-size and per-op-time were not measured in any variant, so the cost side of the marker work remains unverified.

## Open questions

- Does the stacked second-hit gate with MRU landing hold on workloads and capacities other than the LIRS trace at size 2048?
- Did the spare-bit variant actually change the entry layout, and if so, what happens to metadata-size?
- Is the victim-preference idea dead, or does it only fail when the marker is the sole criterion?

## Continuation

Next cycle should build on v001: keep the MRU landing with the second-hit gate, and test it across more workloads and capacities while measuring metadata-size and per-op-time. Drop the never-re-hit eviction-victim preference. Verify the spare-bit variant's diff before spending further cycles on marker relocation.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried three changes around the seen-in-stash marker and stash handling: making the second-hit promotion gate depend on stash occupancy, relocating the marker into a spare tag bit, and changing stash-tier victim selection to the earliest-landed entry in the target bucket. Only v001 is listed as a Winner, but its measured hit-rate is effectively unchanged from its parent, so no variant produced a new measurable gain.

## Patterns

- The occupancy-conditional promotion gate matched its parent on the one measured trace. Pressure-specific gating added no visible value on this workload and capacity.
- Relocating the marker into a spare tag bit left hit-rate exactly unchanged. This is consistent with a pure layout change, but it does not show that any metadata saving was realized.
- Changing stash victim order to earliest-landed produced exactly baseline hit-rate. Eviction-order tweaks in this part of the design have shown no effect so far.
- Across all three variants, no change moved hit-rate, and metadata-size and per-op-time were measured in none. The cost side of every change is unknown.
- Two insights rest on hypothesis text rather than the overlay diff, so the implementations are not confirmed as described.

## Open questions

- Why is v001 a Winner when its insight reports no change from its parent? Is it winning against a baseline, or only carried forward?
- Does any of this cohort's behaviour hold on workloads or capacities other than the LIRS trace at size 2048?
- Did the spare-bit variant actually pack the marker, and if so, what happens to metadata-size?
- Does earliest-landed victim selection ever differ from the current selection on any workload?

## Continuation

Next cycle should not spend more effort on small landing, promotion-gate, or victim-order refinements, since this cohort found none that move hit-rate on the single trace. Instead, it should test the v001 lineage on more workloads and capacities and record metadata-size and per-op-time, so the cost side of the marker work is finally measured. Drop the occupancy-conditional gate and the earliest-landed victim change unless a follow-up shows a measurable effect.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried three changes: a ghost-history fast path for readmitting recently evicted keys, an equivalence-preserving occupancy counter for victim search, and moving probing-tier demotions to the LRU end of their normal-tier bucket. Only v001 is listed as a Winner, yet its own insight reports a slight hit-rate drop on the one measured trace, so the Winner status is not supported by this cohort's evidence.

## Patterns

- Early readmission looks harmful. The ghost-history path let returning keys skip the seen-in-stash wait, and the result was a slight hit-rate loss, which suggests recently evicted keys displaced residents that were still earning their place.
- The two no-change variants agree with each other. The occupancy-counter refactor and the demotion-placement change both matched the parent exactly, so this cohort gives no evidence that victim-search cost or demoted-item placement moves hit-rate on this workload.
- No variant measured per-op-time or metadata-size. The occupancy counter's claimed speedup and the ghost set's memory cost are both untested.
- Three of three insights rest partly on hypothesis text because the overlay diffs were not readable. The implementations are not confirmed as described.

## Open questions

- Why is v001 listed as a Winner when its measured hit-rate is below its parent? Is it winning against a different baseline?
- Does the occupancy counter produce any speedup once per-op-time is measured?
- Is demoted-item placement ever the deciding factor for a victim on any workload or capacity?
- Does the ghost-history cost in metadata justify any hit-rate it might gain on other traces?

## Continuation

Next cycle should not pursue the ghost-history fast path further unless a second workload shows a clear hit-rate gain, since the one measured result was negative. Drop the demotion-placement change, which showed no effect. Keep the occupancy counter only as a cost-side experiment: re-test it with per-op-time and metadata-size recorded, and confirm the overlay diffs before drawing conclusions.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
