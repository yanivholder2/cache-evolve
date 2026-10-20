# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0009

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_14-00-19-782/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort tried folding resident-key membership into the queue-state bit (v001), giving frequency-positive main-queue victims a tail second pass instead of a head reinsert (v002), and returning early on resident hits before the ghost lookup (v003). Both Winners (v001, v003) are cost-side changes that are hit-rate neutral against the parent, so neither is a demonstrated gain; v002 is the only variant that lost hit-rate.

## Patterns

- The two Winners both target per-op-time or metadata-size while leaving eviction order alone. Both held hit-rate flat, which fits that intent, but neither reported the cost dimensions, so their claims remain unmeasured. This gap has now persisted across several cycles.
- Longer retention of warm main-queue residents (v002) lost hit-rate on this trace. This is a new lever, distinct from the promotion-bar and ghost-admission axes, and it suggests the existing head reinsert is not leaving value on the table here.
- The v001 overlay was not inspected this cycle, and the v002 and v003 insights do not confirm their edits either. The same overlay-verification gap raised in prior cycles still applies.
- Evidence is still one trace (lirs workload-a) at one capacity (2048).

## Open questions

- Do the v001 bit-folding and v003 early return reduce per-op-time or metadata-size? Neither can be ranked until a rerun reports both.
- Did the v001, v002, and v003 edits actually land in the built code? Until they are confirmed, the flat and negative results cannot be tied to the intended changes.
- The v003 insight cites v001 as its control. Confirm that the v003 comparison baseline is the intended parent, so that its neutral result is attributed correctly.

## Continuation

Keep the v001 bit-folding and the v003 early return only as cost candidates, and require a rerun that reports per-op-time and metadata-size, plus confirmation that each overlay edit is present, before calling either a win. Drop the tail-reinsert retention change (v002) as a standalone change, since it lost hits on this trace. Do not spend the next cycle on further main-queue retention or reinsert-position tweaks; prioritize build and overlay verification and cost reporting.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort tried replacing queue scans with a resident-key index (v001), removing the ghost-membership mirror set in favor of a linear ghost scan (v002, stacked on v001), and carrying a frequency of 1 into main on ghost re-admission (v003). Both Winners (v001, v002) are hit-rate neutral against the parent and are cost-side changes, so neither is a demonstrated gain; v003 is the only variant that lost hit-rate.

## Patterns

- The two Winners are structurally one line of work: v002 is explicitly equivalence-preserving on top of v001. Their hit-rate results therefore do not count as independent confirmations.
- Both Winners were built to leave eviction order alone and held hit-rate flat, which fits that intent. Neither reported metadata-size or per-op-time, so the O(1) and metadata claims are still unmeasured.
- v002 trades a hash mirror for a linear ghost scan. Its own insight flags a possible per-op-time cost on the miss path, so the cost direction it claims may not hold.
- Ghost re-admission has now been pushed in a third way. Widening the ghost window (Cycle 0003), routing ghost hits through the small queue (Cycle 0004), and giving re-admissions an extra main-queue pass (v003) all lost or failed to help. Ghost-admission levers look unproductive on this trace.
- The v001 and v002 insights do not confirm that their intended edits are present in the built code. The overlay-verification gap from prior cycles still applies.
- Evidence is still one trace (lirs workload-a) at one capacity (2048).

## Open questions

- Does the resident-key index (v001) reduce per-op-time, and at what metadata cost? Only a rerun reporting both can settle it.
- Does removing the ghost mirror (v002) help metadata-size enough to offset a linear miss-path scan? Its cost direction is unknown until per-op-time is measured.
- Did the v001 and v002 edits land in the built code? Their flat hit-rate results cannot be tied to the intended changes until this is confirmed.

## Continuation

Treat the resident-key index (v001) as the lead cost candidate, and require a rerun that reports per-op-time and metadata-size plus confirmation of the overlay edit before calling it a win. Hold the ghost-mirror removal (v002) until v001 is measured, since its linear scan may regress per-op-time. Drop giving ghost re-admissions a frequency of 1 (v003) as a standalone change. Do not spend the next cycle on further ghost-admission or frequency tweaks on this trace; prioritize build and overlay verification and cost reporting.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cohort tried a cheaper hasher on the resident index (v001), moving small-queue second-hit entries to the front of main earlier (v002), and a counting pre-filter in front of the ghost linear scan (v003). All three Winners are hit-rate neutral against the parent, so none is a demonstrated gain; the cohort is cost-side work that has not yet been measured on cost.

## Patterns

- All three Winners held hit-rate flat, which fits their intent: each changes how work is done or when a warm key moves, not which keys are evicted. None reported per-op-time or metadata-size, so every cost claim remains unmeasured.
- Two of the three edits (v001 hasher, v003 pre-filter) are constant-factor or metadata changes on the miss path. The ghost linear scan is still untouched by v001, so any gain from it is bounded.
- The promotion-timing change (v002) showed that moving an already-warm key earlier does not alter residency. Promotion-timing levers need to change the eviction decision itself to matter.
- v003's implementation diverges from its hypothesis: counters are wider than specified, and the key is hashed on every ghost push, pop and re-admission, so the eviction path may slow even if skipped scans save time. The cost direction is genuinely uncertain.
- Overlay edits were not confirmed for any variant this cycle, the same verification gap carried since earlier cycles.
- Evidence is still one trace (lirs workload-a) at one capacity (2048).

## Open questions

- Does the hasher swap (v001) lower per-op-time on get and put, and does the ghost scan cap that gain?
- Does the ghost pre-filter (v003) save more on skipped scans than it costs in per-push hashing and wider counters? Only a rerun reporting per-op-time and metadata-size can answer this.
- Did the v001 and v003 edits land as described in the built code, and did v002's promotion edit land at all?

## Continuation

Carry the hasher swap (v001) and the ghost pre-filter (v003) forward only as cost candidates, and require a rerun that reports per-op-time and metadata-size plus confirmation of each overlay edit before calling either a win; test v003 with the counter width the hypothesis specified. Drop the earlier promotion of second-hit small entries (v002) as a standalone change, since it moved timing without changing hits. Do not spend the next cycle on further promotion-timing tweaks; prioritize build and overlay verification and cost reporting.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
