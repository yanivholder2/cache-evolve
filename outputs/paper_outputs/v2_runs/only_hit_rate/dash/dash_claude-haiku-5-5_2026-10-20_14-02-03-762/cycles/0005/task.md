# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0005

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/dash_2026-10-20_14-02-03-762/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort tried three changes around stash promotion and foreign-eviction placement: gating stash promotion on a second hit, making the stash-hit landing slot depend on target-bucket occupancy, and restoring the target retry for foreign evictions only when the stash is full. Only v001 is listed as a Winner, and its measured hit-rate matched its parent, so the win is inherited rather than new.

## Patterns

- The one clear hit-rate gain came from admission filtering. Requiring a second hit before a stash item is promoted helped on the LIRS trace, which suggests one-off stash hits were displacing resident items.
- Refinements of the landing slot changed nothing measurable. The occupancy-conditional landing matched its parent, so the cohort gives no evidence that the full-bucket case it targets is common on this workload.
- Gating the direct-to-stash shortcut on free stash capacity recovered only a small part of the hit-rate lost by the unconditional shortcut. That supports the view that stash pressure is what made the removed retry valuable.
- Every variant was judged on hit-rate alone. Per-op-time and metadata-size were measured in none of them, so no cost of the promotion filter or its marker is known.
- Several insights rest on the hypothesis text rather than the overlay diff, so the implementations are not confirmed as described.

## Open questions

- Is v001 a Winner against the baseline, or only against its parent v002? Its own insight reports no change from that parent.
- Does the second-hit promotion filter hold on workloads other than the LIRS trace?
- Does the one-bit seen-in-stash marker stay within existing entry metadata, and what does it cost in per-op-time?
- Is the occupancy-conditional landing ever triggered on any workload?

## Continuation

Next cycle should build on the second-hit promotion filter: test it on more than one workload and measure per-op-time and metadata-size alongside hit-rate. Drop the occupancy-conditional landing refinement, since it showed no effect, and treat the capacity-gated direct-to-stash path as a secondary lever to revisit only if the promotion filter's cost is measured.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
