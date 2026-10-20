# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0006

## Winners

- v002
- v001
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0006/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0006/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0006/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0006/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
