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
