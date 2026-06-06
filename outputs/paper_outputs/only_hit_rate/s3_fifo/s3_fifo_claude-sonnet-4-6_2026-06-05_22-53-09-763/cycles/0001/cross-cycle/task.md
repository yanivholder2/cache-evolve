# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants targeted the balance between the small, main, and ghost queues; only the pure capacity reallocation (shrinking the small queue from 25% to 10%) won, while both attempts to loosen admission quality — lowering the promotion threshold and widening the ghost window — hurt hit rate.

## Patterns

- **Resident capacity beats admission breadth.** The winning change gave more slots to main without touching any admission logic. Both losing changes expanded what could enter main (more promotions, more ghost hits fast-pathed in) and degraded performance, suggesting main queue quality is more sensitive to entry criteria than to raw size pressure.
- **The main queue's second-chance mechanism is the critical bottleneck.** When lower-quality entries flooded in (v002, v003), second-chance eviction could not shed them fast enough to protect genuinely hot items. The frequency threshold (`freq > 1`) and a right-sized ghost window act as upstream guards that second-chance alone cannot replace.
- **Ghost queue value is precision, not coverage.** Extending the ghost window introduced incidental re-accesses that bypassed the small-queue filter. A longer history does not improve signal quality for this trace; it introduces noise from distant, non-recurring accesses.

## Open questions

- Is 10% the right floor for small queue size, or does further shrinking (e.g., 5%) keep improving — and does the answer change under scan-heavy or more bursty phases?
- Could a *smaller* ghost queue (below the baseline `main_size` cap) improve precision further, given that a larger one demonstrably hurts?
- Does the winning small=10% configuration shift the optimal promotion threshold, or does `freq > 1` remain correct regardless of small-queue sizing?

## Continuation

Next cycle should build on v001's winning configuration (small=10%) and probe in two directions: first, whether fine-grained small-queue sizing around 10% (e.g., 8%, 12%) yields further gains; second, whether the ghost queue has an optimal size below the current baseline cap — the evidence from v003 suggests shrinking it is worth testing. Attempts to relax the small-to-main promotion threshold should be parked unless paired with a compensating mechanism to protect main-queue quality.


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
