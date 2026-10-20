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
