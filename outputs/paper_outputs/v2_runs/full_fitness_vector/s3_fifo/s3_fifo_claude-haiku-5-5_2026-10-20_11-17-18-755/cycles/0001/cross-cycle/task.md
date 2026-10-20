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
