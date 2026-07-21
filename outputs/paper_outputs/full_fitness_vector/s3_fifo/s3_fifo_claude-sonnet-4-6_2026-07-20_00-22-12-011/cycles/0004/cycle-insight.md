# Cycle 0004 Insight

## Cohort outcome

This cycle tested three hypotheses: a bounded second-chance scan cap in `evict_main()` (v001), further small-queue shrinkage from 1/20 to 1/40 (v002), and a step-2 frequency decrement to accelerate victim selection (v003). None of the three improved on the prior baseline without a corresponding regression, and all three variants independently surfaced the same underlying bottleneck: a linear `position()` scan in `get()` and `put()` that was not targeted by any of these changes.

## Patterns

- **The bounded-scan hypothesis was falsified**: The cycle 0003 continuation proposed capping the second-chance loop to recover hit-rate quality from v001 of that cycle. Testing revealed the opposite: on Zipfian workloads the unbounded scan already terminates in the first few iterations because cold entries naturally accumulate at the tail of main, so a fixed cap delivers no churn reduction while introducing forced eviction of still-hot entries beyond the budget.
- **The `position()` scan is the dominant latency cost**: Both v001 and v002 independently named a linear key-lookup scan in `get()` and `put()` — not the second-chance eviction loop — as the dominant per-operation overhead. This cost grows with main-queue depth, which means every slot shifted from small to main (v002) worsens it, and every approach that leaves main large leaves this hotspot untouched.
- **Small-queue shrinkage has entered a diminishing-returns regime**: The monotone hit-rate improvement from halving the small-queue fraction continued at 1/40, but for the first time the latency penalty from a longer main queue was material rather than negligible — the two effects are approaching a crossover as main approaches full capacity.
- **Frequency acceleration amplifies eviction work rather than reducing it**: Draining frequency faster (step=2) caused moderately-warm entries to exit main prematurely, increasing the rate of ghost-hit-driven `evict_main()` calls. Total eviction cost rose even as individual scan lengths shortened — scan-length reduction and total-eviction-work reduction are decoupled and can oppose each other.

## Open questions

- Can the `position()` linear scan in `get()` and `put()` be replaced with an O(1) lookup via an intrusive node structure, a side-map, or a hashmap into the deque — and would eliminating it resolve the latency penalty that now shadows both the small-queue sizing lever and the eviction-loop improvements?
- Is there a minimum small-queue fraction below which the main-queue `position()` cost permanently dominates the hit-rate gain, and is 1/20 already near that threshold?

## Continuation

The `position()` linear scan in `get()` and `put()` has now been identified by two independent variants as the primary latency bottleneck; it should be the sole focus of the next cycle's hypotheses. The small-queue fraction should be held at 1/20 — the clean, unambiguous win from cycle 0003 — rather than pushed to 1/40 where scan overhead begins to cancel the hit-rate gain. Do not revisit the bounded second-chance cap (falsified) or frequency step acceleration (falsified); both hypotheses have been conclusively ruled out on Zipfian workloads.
