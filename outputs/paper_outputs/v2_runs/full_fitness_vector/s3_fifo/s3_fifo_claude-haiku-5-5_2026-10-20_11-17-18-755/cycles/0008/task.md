# Task: cycle-insight-writer

Cycle: 0008
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0008

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0008/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0008/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0008/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0008/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0008/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0008/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
