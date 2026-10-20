# Cycle 0008 Insight

## Cohort outcome

The cohort tried three edits: replacing the linear resident scans with an id-keyed index, swapping the index hasher for a fixed multiply-shift hash, and lowering the frequency ceiling from 3 to 2. The id-keyed index is the clear Winner, with direct get and put speedups and no hit-rate change. The hasher swap shrank the container but made gets slower. The frequency ceiling lost hit rate on its one workload and offers no offsetting gain, so its Winner label is not supported by its own insight.

## Patterns

- Index-based lookup is now a repeated direction (Cycles 0005 through 0008). This cohort's id-keyed index has the most direct get and put rationale so far: the resident path becomes one lookup instead of two linear scans. Its speedups still need a baseline rerun before they are credited.
- Container-level footprint remains the unit that matters. The index grew the container, the hasher swap shrank it, and neither changed `Entry`.
- Changing the hash function did not reduce get cost. The get path got slower under the multiply-shift hasher, which suggests hashing is not the lever on the hit path once the index exists.
- Lowering the frequency ceiling to 2 cost hit rate on workload-a at 2048 and saved no metadata. The third frequency level appears to earn its keep, at least on this trace.
- Get-path timing moved in variants whose get logic was unchanged, matching the shared-noise pattern from earlier cycles.

## Open questions

- Does the id-keyed index keep its get and put speedups under a baseline rerun and across traces and capacities? Does its extra container state matter at larger sizes?
- Is the hasher variant's get regression real, or layout and timing noise? Does its container shrink hold at other sizes?
- Does the frequency ceiling of 3 hold on other traces, or is the hit-rate loss specific to workload-a at 2048?
- Do the index and the Cycle 0007 counter relocation and ghost window compose cleanly, given they touch overlapping state?

## Continuation

Next cycle should treat the id-keyed index as the lead candidate: validate its get and put speedups with a baseline rerun and on more traces before crediting them. Keep the frequency ceiling at 3; do not lower it on current evidence. Drop the multiply-shift hasher as a get-path lever. The frequency-ceiling variant should not be counted as a Winner on this cohort's evidence.
