# Cycle 0002 Insight

## Cohort outcome

The cohort tried three changes around stash promotion and foreign-eviction placement: gating stash promotion on a second hit, making the stash-hit landing slot depend on target-bucket occupancy, and restoring the target retry for foreign evictions only when the stash is full. Only v001 is listed as a Winner, and its measured hit-rate matched its parent, so the win is inherited rather than new.

## Patterns

- The one clear hit-rate gain came from admission filtering. Requiring a second hit before a stash item is promoted helped on the LIRS trace, which suggests one-off stash hits were displacing resident items.
- Refinements of the landing slot changed nothing measurable. The occupancy-conditional landing matched its parent, so the cohort gives no evidence that the full-bucket case it targets is common on this workload.
- Gating the direct-to-stash shortcut on free stash capacity recovered only a small part of the hit-rate lost by the unconditional shortcut. That supports the view that stash pressure is what made the removed retry valuable.
- Every variant was judged on hit-rate alone. Per-op-time and metadata-size were measured in none of them, so no cost of the promotion filter or its marker is known.
- Several insights rest on the hypothesis text rather than the overlay diff, so the implementations are not confirmed as described.

## Open questions

- Is v001 a Winner against the baseline, or only against its parent v002? Its own insight reports no change from that parent.
- Does the second-hit promotion filter hold on workloads other than the LIRS trace?
- Does the one-bit seen-in-stash marker stay within existing entry metadata, and what does it cost in per-op-time?
- Is the occupancy-conditional landing ever triggered on any workload?

## Continuation

Next cycle should build on the second-hit promotion filter: test it on more than one workload and measure per-op-time and metadata-size alongside hit-rate. Drop the occupancy-conditional landing refinement, since it showed no effect, and treat the capacity-gated direct-to-stash path as a secondary lever to revisit only if the promotion filter's cost is measured.
