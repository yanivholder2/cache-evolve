# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried three changes to how items move between the probing, stash, and normal tiers: where a stash-promoted item lands in its target bucket, whether a probing hit re-inserts the item, and whether foreign evictions skip the retry in their true target. Only v002, the change to the landing slot for stash hits, won; the other two regressed on hit-rate and were not winners.

## Patterns

- Landing position matters more than movement. Moving a re-referenced stash item to the MRU end of its target bucket was the only change that helped, while the two changes that added movement or removed a retry both hurt hit-rate.
- Keeping residency in the probing tier appears valuable. Removing and re-inserting probing hits pushed proven-hot keys back through target competition and lost ground.
- Shortening the demotion path alone does not pay off. Skipping the target retry for foreign evictions lowered the one measured dimension, and the predicted per-op-time gain was never recorded.
- The winner's evidence is thin. Its only measured dimension is hit-rate, on a single workload, so the win is a hit-rate result, not a demonstrated overall improvement.
- Rationale is inconsistently documented around the winning change. The Front/Back default is described differently in different places, which makes the intent of v002 harder for later cycles to verify.

## Open questions

- Does the MRU landing for stash hits hold on workloads other than the LIRS trace used here?
- Do per-op-time and metadata-size move at all under any of these changes? No variant measured them.
- Would the removed target retry pay for itself if the shorter eviction path is paired with better stash capacity handling?

## Continuation

Next cycle should build on v002: vary the stash-hit landing slot further and measure it across more workloads, plus per-op-time and metadata-size. Drop the probing-hit remove-and-reinsert idea and the direct-to-stash eviction change unless a follow-up explicitly measures the costs they were meant to offset.


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

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
