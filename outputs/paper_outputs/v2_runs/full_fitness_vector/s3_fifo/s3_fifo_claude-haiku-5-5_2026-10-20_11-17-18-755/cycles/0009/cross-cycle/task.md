# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort tried three small, localized edits: a promotion-threshold change in the small queue, a hash index for ghost membership, and a fixed budget on main-queue second-chance rotations. Only the ghost-index and scan-cap changes show a clear, attributable mechanism (put-path cost); the promotion change shows no attributable gain and a hit-rate cost, so its Winner status is weakly supported.

## Patterns

- Changes that touched only the put path (ghost lookup, eviction scan cap) were the ones with an explained benefit; both left get-path timings unexplained, which points to measurement noise rather than the edits.
- Structural and bounding changes trade cost for hit rate in predictable ways: the scan cap pays in forced evictions of hot entries, and the ghost index adds fixed struct overhead.
- Hit-rate is unchanged by the pure data-structure swap, which confirms that change left policy intact.
- The promotion-threshold change moved singly-reused keys into main earlier and displaced useful residents; no latency benefit was attributable to it.

## Open questions

- Is the get-path latency shift seen across variants real, or a shared environment or measurement artifact? It is unexplained by every variant's diff.
- How often does the 8-rotation budget actually run out under the evaluated workloads, and how much hit rate does that cost in aggregate?
- Does the ghost queue's O(n) pop-back and position-based removal still dominate once the membership test is O(1)?

## Continuation

Pursue the put-path directions: ghost-queue maintenance cost and bounded eviction, measured with a rerun to separate noise from effect. Drop further tuning of the small-queue promotion threshold (`freq > 0`), since it showed no attributable gain and a hit-rate loss on the workload tested. Re-measure the get path before crediting any get-latency change to a variant.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

The cohort tried three localized policy and sizing edits: a sequence-tagged ghost queue, a bounded lowest-frequency victim choice after the second-chance budget runs out, and a larger small-queue share of capacity. Only the small-queue resize and the victim choice show a hit-rate effect, and both gains are small and drawn from a single trace. The ghost-queue change kept hit rate but worsened put cost, so it is the weakest of the three.

## Patterns

- Hit-rate movement was small and came from one trace at one capacity in each case, so none of the three gains is yet robust.
- The ghost-queue change preserved policy exactly but added per-eviction bookkeeping on the put miss path, which confirms that the ghost `position` removal was not the dominant put cost.
- Get-path latency shifted in more than one variant even though `get()` was not modified. This matches the open question from Cycle 0001 and points to shared measurement noise or environment drift, not to any variant's diff.
- Two of the three insights note that the attribution rests on the hypothesis description because the overlay source could not be inspected. Mechanism claims are therefore weaker than the Winner labels imply.
- Winner status is not uniform: one variant gained no hit rate and worsened put latency, so its place among the Winners is hard to justify on the evidence here.

## Open questions

- Does the small-queue share increase hold across other traces and capacities, or is the hit-rate gain specific to the workload and size tested?
- Does the lowest-frequency victim choice keep its hit-rate recovery and put-tail benefit when the rotation budget runs out more often?
- Where is the dominant put miss-path cost, if not the ghost `position` scan?
- Is the get-path shift reproducible on rerun, and does it appear with no code change at all?

## Continuation

Next cycle should validate the small-queue resize and the post-budget victim choice across more traces and capacities before treating either as a Winner. Drop further ghost-queue sequence-tagging, since it added put cost without a policy gain. Re-measure the get path with a baseline rerun before crediting any get-latency change to a variant.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort tried three edits: stacking a lowest-frequency victim choice on the larger small queue, frequency aging on main-queue rotation, and a fingerprint-keyed ghost queue. Only the fingerprint ghost shows a clear, attributable benefit (put-path latency). The other two show no hit-rate gain and no attributable benefit, so their Winner labels are weakly supported.

## Patterns

- The one clear win came from the put miss path: replacing stored ghost keys with fingerprints removed per-eviction key handling, and that change is what moved put latency.
- The fingerprint ghost's cost is in memory and tail risk, not hit rate: the cache-level bookkeeping grew, and ghost hits now scan the ghost queue linearly.
- Victim choice did not compose with the larger small queue from Cycle 0002. Once the small queue holds the larger share, the rotation budget likely runs out less often, so the victim change has little to act on.
- Frequency aging left hit rate unchanged on the one trace reported, so its effect is untested rather than shown to be neutral.
- Two of three variant insights report empty "Helped" sections and rest on hypothesis text because overlay source could not be inspected. Mechanism claims remain unverified.
- Get-path timing moved again without any get-path edit, matching the shared-noise pattern from Cycles 0001 and 0002.

## Open questions

- Does the fingerprint ghost keep its put-latency gain on more traces and capacities, and does the linear ghost-hit scan show up in tail latency at larger sizes?
- Does the victim choice ever help once the small queue is enlarged, or is it only useful in a regime where the rotation budget is exhausted often?
- Does frequency aging change eviction on a trace where the second-chance pass actually runs long?
- Is the get-path shift real, and is it reproducible with no code change?

## Continuation

Next cycle should validate the fingerprint ghost on more traces and capacities, and check its ghost-hit tail latency before treating it as a settled Winner. Drop the stacked victim choice on top of the 20% small queue, since it lost hit rate on the trace tested. Hold frequency aging until a trace shows it changing evictions. Re-run a get-path baseline before crediting any get-latency change.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried three metadata and sizing edits: enlarging the ghost queue to full cache capacity, storing ghost entries as fingerprints, and moving the per-entry frequency counter into a parallel side array. Only the fingerprint ghost shows a clear, attributable benefit (put-path cost), and it repeats the direction from Cycle 0003. The other two are regressions or trade-offs, so their Winner labels are hard to justify on this evidence.

## Patterns

- The fingerprint ghost again moved put-path cost without changing hit rate, which is the same result Cycle 0003 reported. Its recurring cost is container-level metadata (an added hash set), not per-entry size.
- Ghost capacity is not a free dial: enlarging it raised the number of keys re-admitted to main without a matching reuse gain, and hit rate fell on the trace tested.
- Shrinking the stored entry did not shrink the structure. The side-array split saved bytes in the entry but grew the container, so footprint was a net loss.
- The side-array split is the first variant in this series whose get-path timing shift has a direct code-level rationale (an extra indexed read per hit). That is still an unverified claim, but it is different from the unexplained get-path noise seen in earlier cycles.
- Hit rate was unchanged for the policy-preserving edits and dropped only where the ghost was enlarged.
- Two of three variant insights could not inspect overlay source and rest on hypothesis text and fitness deltas, so mechanism claims remain provisional.

## Open questions

- Does the fingerprint ghost's put-latency gain hold across traces and capacities, and is its container-size cost acceptable at larger sizes?
- Is there a ghost capacity between main-sized and full-capacity that keeps any reuse benefit without the hit-rate loss?
- Is the get-path increase from the side-array split real and reproducible, and does it persist when the counter is kept in the entry?

## Continuation

Next cycle should validate the fingerprint ghost on additional traces and capacities, since it is now the only edit with a repeated, attributable benefit; drop any further ghost enlargement toward full capacity. Drop the counter side-array split unless a future variant measures whole-structure footprint and shows the get-path cost is absent. Re-run a get-path baseline before crediting any get-latency change.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried three edits, all aimed at metadata and lookup cost rather than eviction policy: capping the ghost population below main-queue size, replacing per-queue key scans with id-indexed hash lookups, and storing ghost entries as fingerprints in a hash set. All three were labeled Winners. The fingerprint ghost and the index change have the clearest cost rationale; the ghost cap is the only one with a hit-rate movement, and that movement rests on a single trace.

## Patterns

- The fingerprint ghost is the third cycle in a row to show a put-path benefit with no hit-rate change (after Cycles 0003 and 0004). Its gain is again on the put side, and its cost is container-level bookkeeping, not per-entry size.
- Ghost size moved in opposite directions across cycles with opposite results. Enlarging the ghost to full capacity (Cycle 0004) lost hit rate; capping it at half the main queue here gained a small amount. On the evidence so far, a smaller ghost population looks closer to what the workload needs.
- The index change removed the per-put resident-key scan and the per-hit queue walk. Hit rate was unchanged because eviction and promotion logic were untouched, and the cost is fixed container state rather than per-entry bytes.
- Get-path timing moved in one variant whose get code is unchanged, which matches the shared-noise pattern from earlier cycles. The index change is the first get-path edit with a direct speedup rationale, but it still needs a baseline rerun before the credit is taken.
- Whole-structure footprint is the right unit for these changes. Each Winner grew the container while leaving `Entry` alone, so per-entry size is no longer a useful proxy.
- The three edits touch overlapping state (ghost storage in two of them), so their gains may not add together. None was tested in combination.

## Open questions

- Does the fingerprint ghost's put-latency gain hold across traces and capacities, and does its hash-set cost stay acceptable at larger sizes?
- Does the half-main ghost cap keep its small hit-rate gain on other traces and capacities, and does it still help once the ghost is stored as fingerprints?
- Does the id-indexed lookup keep its get and put speedup under a baseline rerun, and does the extra index state matter at larger sizes?

## Continuation

Next cycle should validate the fingerprint ghost on more traces and capacities, since it now has three cycles of consistent put-side benefit. Test the half-main ghost cap on additional workloads, and try it in combination with the fingerprint ghost before treating either as settled. Drop further ghost enlargement toward full capacity. Re-run a get-path baseline before crediting the index change's get speedup.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tried three edits: a half-main cap on the fingerprint ghost, merging the two per-queue indexes into one id-keyed map, and raising the small-queue share on its own. The index merge is the cleanest Winner, with a container-size and put-cost benefit and no hit-rate change. The ghost cap is the only edit with a hit-rate movement, and it is paid for on the put path. The share change has weak support.

## Patterns

- The ghost cap combined with fingerprint storage gave a small hit-rate gain, but its per-eviction drain adds put-side cost. This erodes the put-latency benefit the fingerprint ghost showed in earlier cycles, so the two effects need to be separated.
- The index merge reduced container state and put cost while leaving eviction, ghost, and promotion logic untouched. Hit rate was unchanged. Its only visible cost is a small decode on the get hit path, which is within noise.
- The small-queue share change alone left hit rate unchanged on the one workload reported. This is consistent with Cycle 0003: the earlier hit-rate gain came from the victim-choice change bundled with the larger share, not from the share itself. Its latency movement is modest and comes from a single run.
- Get-path timing moved in variants whose get code was not changed, matching the shared-noise pattern from Cycles 0001 through 0005.
- Container-level footprint is again the unit that matters. Each edit changed the container, not `Entry`.

## Open questions

- Can the ghost-cap drain be amortized, for example by compacting in batches, so the hit-rate gain survives without the put-side cost?
- Does the index merge's put gain reproduce under a baseline rerun, and does the tagged-slot decode matter at larger sizes?
- Is the small-queue share change's latency shift real, or is it single-run noise?

## Continuation

Next cycle should pursue the index merge as the strongest candidate: it has no hit-rate cost and a structural benefit, so validate it on more traces before treating it as settled. Keep the half-main ghost cap only if its drain can be made amortized; otherwise drop it. Do not credit the standalone small-queue share change as a Winner without repeated runs. Re-run a get-path baseline before crediting any get-latency change.


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

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
