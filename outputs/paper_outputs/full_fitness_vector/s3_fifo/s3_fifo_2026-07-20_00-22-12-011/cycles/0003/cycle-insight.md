# Cycle 0003 Insight

## Cohort outcome

This cycle probed three independent axes: further small-queue shrinkage (v002), O(1) victim selection in `evict_main()` (v001), and a lower ghost-promotion threshold (v003). The cleanest win was v002, which continued the monotone small-queue reduction trend with no downside; v001 delivered a large latency gain at the cost of hit-rate; and v003 backfired, confirming the ghost channel is a necessary quality gate.

## Patterns

- **Small-queue shrinkage has not hit its floor**: Halving the fraction again (1/10 → 1/20) improved both hit-rate and get latency without any observed penalty — the monotone trend begun in cycle 0001 is still running clean on Zipfian workloads.
- **Deque churn, not scan length, was the latency driver in `evict_main()`**: Two-random-choices eliminated the `pop_back`/`push_front` memory-layout churn and halved get p99, but the associated loss of frequency-drain protection allowed hot entries to be discarded on a single bad draw — the hit-rate cost was material, not noise. The complexity class (O(n) vs O(1)) and the eviction *policy* were coupled; this variant untangled them in a way that confirmed each has independent value.
- **The ghost channel is a dual gate — frequency *and* demand confirmation**: Promoting at `freq > 0` short-circuited the ghost window's role as a re-request filter, flooding main with once-warm entries that lacked strong recency evidence. The paper's `freq > 1` threshold is not conservative bookkeeping; it enforces that main receives only keys that have been re-requested *after* ghost-listing, not merely re-accessed while still in small.
- **All three variants landed in the Winner list despite mixed fitness signals**: v001 hurt hit-rate and v003 showed no benefit alongside a hit-rate regression, indicating the evaluation fitness function may weight latency or aggregate score in ways that allow hit-rate regressions to be offset. This should be treated as a signal about fitness-function sensitivity, not as endorsement of those changes in isolation.

## Open questions

- Can a bounded second-chance scan — e.g., a fixed cap of 8–16 candidates rather than full O(n) traversal — recover most of v001's latency gain while preserving the frequency-drain protection that kept the hit-rate intact?
- Is there a floor to the small-queue shrinkage benefit, and does the ghost deque's expanded capacity (which tracks `main_size`) begin to inflate tombstone backlog pressure as `main_size` grows toward the full capacity?
- The winner designation for v003 (no benefits, one hurt) is anomalous — does the fitness function include a latency dimension that v003 happened to pass, or is this a threshold artifact worth investigating?

## Continuation

v002 (1/20 small-queue fraction) is the clear carry-forward winner and should become the new baseline; it is a clean, risk-free improvement on every measured axis. The next cycle's primary hypothesis should be a bounded second-chance scan in `evict_main()` — testing whether a shallow cap (rather than full O(n) or pure O(1) random) recovers hit-rate quality while still eliminating most of the deque-churn latency that v001 identified. Do not revisit the `freq > 0` promotion threshold; v003 conclusively demonstrated that the ghost channel's demand-confirmation function is load-bearing and the paper's `freq > 1` rule should remain fixed.
