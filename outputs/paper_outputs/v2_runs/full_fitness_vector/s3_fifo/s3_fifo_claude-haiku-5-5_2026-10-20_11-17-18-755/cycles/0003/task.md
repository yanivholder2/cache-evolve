# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0003

## Winners

- v001
- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0003/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0003/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0003/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_11-17-18-755/cycles/0003/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
