# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0007

## Winners

- v001
- v003
- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0007/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
