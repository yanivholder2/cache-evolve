## Recurring wins

- No hit-rate gain has appeared in either cycle. Winners so far are neutral against the parent, not improvements.
- The two-hit small-queue promotion bar is the only change with a measured effect. Relaxing it to one hit cost hits by displacing main-queue residents (cycle 0001), so the stricter bar looks worth keeping.
- A cost-only refactor of ghost-membership lookup preserved eviction order (cycle 0002). It is a candidate for lower cost, but that is unverified until per-op-time and metadata-size are reported.

## Recurring failures

- Single-constant tweaks on this S3-FIFO trace show no signal. Ghost re-admission routing and small-queue sizing were flat in cycle 0001, and the main-queue frequency cap was flat in cycle 0002. Each axis was tested only with one constant change.
- Cost metrics have been missing in every cycle. Neither metadata-size nor per-op-time has been reported, so no variant can be ranked against the baseline on cost.
- Results so far come from one trace (lirs workload-a) at one capacity (2048). Null results may reflect that setting rather than the policy.
- A layout change (cycle 0002, v003) did not reach the overlay, and its hit-rate drop cannot be attributed to the idea. Mismatched builds have now blocked one hypothesis from being tested.

## Open questions

- Does the ghost-lookup refactor reduce per-op-time, and what does it cost in metadata?
- Is the v003 hit-rate drop real? Does it reproduce on a clean build with the layout change actually applied?
- Do the flat results for routing, sizing, and frequency hold at other capacities or workloads? Or is this trace insensitive to them?
- Would a larger structural change to small-queue sizing, rather than a one-constant tweak, move hit-rate?

## Per-policy notes

- S3-FIFO (only policy family in the ledger):
  - Small-queue promotion: the two-hit bar helps; relaxing it hurts on this trace.
  - Ghost routing and small-queue sizing: no measured effect so far; sizing was tested only with a single-constant change.
  - Main-queue frequency cap: no effect on this trace, which suggests few main-queue residents reach the cap.
  - Ghost-membership lookup: a cost-only refactor that kept eviction order; cost benefit is unmeasured.
  - Layout change: untested, pending a verified build.
