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
