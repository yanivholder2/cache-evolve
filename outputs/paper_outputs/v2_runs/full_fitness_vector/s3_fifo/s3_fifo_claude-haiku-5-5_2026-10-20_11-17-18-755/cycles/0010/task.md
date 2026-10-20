# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0010

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort tried three metadata and bookkeeping edits: moving the frequency counter out of the queue entry into the index record, replacing eager ghost removal with a push-count window and batched pruning, and replacing the per-queue position scans with a single id-keyed index. All three were labeled Winners, but the evidence is uneven: the ghost window is the only edit with a hit-rate movement, and the index and counter edits show no hit-rate change.

## Patterns

- The ghost window is the first edit in this series to address the drain cost directly. Expiry is batched and runs off the per-eviction path, which answers the open question from Cycle 0006 about amortizing the ghost drain. Its hit-rate gain is small and rests on a single trace.
- Index-based lookup is now a repeated direction. Cycles 0005 and 0006 moved toward id-keyed maps, and this cohort's index variant does the same again. Its get and put speedups have a plausible code-level rationale, but they are unverified and still need a baseline rerun before they are credited.
- The frequency-counter relocation changed `Entry` but not hit rate. Its container-size effect is not clearly reported, so the claim that the change is free in footprint is not yet established.
- Container-level footprint remains the unit that matters. Each Winner changed the container, and none changed `Entry` in a way that could be measured as a net saving.
- The metadata edits and the index edit overlap in the state they touch. The index variant and the counter relocation may both reshape the same record, and no variant was tested in combination with another, so their gains may not add.
- Get-path timing moved in at least one variant whose get logic was unchanged, which matches the shared-noise pattern from earlier cycles.

## Open questions

- Does the ghost window's hit-rate gain hold on other traces and capacities, and is the window size close to the strict cap the way the variant claims?
- Does the frequency counter's container-size cost stay at zero under a whole-structure measurement, or does the index record's extra byte show up at larger sizes?
- How much of the index variant's put-side gain is due to the index itself, as opposed to the inherited ghost behavior? An ablation with the index removed would separate them.
- Do the counter relocation and the index change compose, or do they conflict on the same record?

## Continuation

Next cycle should validate the ghost window on additional traces and capacities, since it is the only edit with a hit-rate movement and it addresses the drain cost raised in Cycle 0006. Treat the index lookup as established direction: validate its speedup with a get-path baseline rerun rather than crediting it again as new. Test the frequency-counter relocation only together with a whole-structure footprint measurement, and do not count it as a Winner on the `Entry` size change alone.


### Cycle 0008

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

The cohort tried three edits: periodic halving of main-queue frequency counters, replacing the fingerprint ghost's table with a flat linear-probing table, and capping the fingerprint ghost at half the main queue's capacity. All three are labeled Winners, but the variant insights do not support that label well. Only the ghost cap shows a hit-rate gain, and it pays for it on the put path. The aging edit lost hit rate, and the flat table shows only a put-tail regression.

## Patterns

- Ghost-side changes are the only group with any hit-rate movement this cycle. The ghost cap gained slightly on the one reported trace, and this is the same ghost-drain thread raised in Cycles 0006 and 0007, now approached from the capacity side rather than by amortizing expiry.
- Aging the main-queue frequency counters hurt hit rate on workload-a at 2048. Cooling hot entries made them evictable sooner, which suggests the existing second-chance rotation already keeps the right entries and that the frequency signal is worth more than it discards.
- The flat ghost table left hit rate exactly unchanged, as predicted, but its predicted put-miss median gain and footprint reduction did not appear. The one measured effect is a put-tail regression from probe work on full-ghost inserts. Its container-size probe is blind to heap-allocated table bytes, so the footprint question is still open rather than answered.
- Put-side cost is the common thread. Every edit that moved a put metric moved it the wrong way or within noise, and none of them produced a clear put-latency gain.
- Get-path timing moved in variants whose get logic was unchanged, matching the shared-noise pattern from Cycles 0001 through 0008. None of these should be credited as get-path effects.
- The container-level footprint remains the unit that matters for the aging and cap edits. Neither changed `Entry`, so the summary metadata figure understates their cost.

## Open questions

- Does the ghost cap's small hit-rate gain hold on other traces and capacities, and can its insert-path expiry cost be reduced by batching, as Cycle 0007's window did?
- Does a much longer aging period, or halving only entries unhit for a full window, recover hit rate without the loss seen here?
- Does the flat ghost table's footprint gain exist once heap bytes are counted, and is its put-tail regression a probing cost that a different load factor or deletion scheme would avoid?
- Why were all three variants labeled Winners when none shows a clear, unambiguous gain on its own evidence? The labeling should be checked before any of them is carried forward.

## Continuation

Next cycle should not treat any of this cohort as settled. Keep the ghost cap as the only live hit-rate candidate, and validate it on more traces and capacities before crediting it, while checking its put-side cost. Drop the main-queue frequency aging in its current form; revisit only with a much longer period or a per-entry unhit window. Do not credit the flat ghost table on its container-size reading: measure ghost heap bytes and the put-miss path first, and treat the put-tail regression as the open risk.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
