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
