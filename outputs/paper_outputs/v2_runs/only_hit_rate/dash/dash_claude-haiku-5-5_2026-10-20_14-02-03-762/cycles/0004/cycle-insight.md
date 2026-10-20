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
