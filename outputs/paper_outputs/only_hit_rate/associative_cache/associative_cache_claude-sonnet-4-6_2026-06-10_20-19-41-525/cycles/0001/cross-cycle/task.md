# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort probed three independent axes — bucket geometry, per-item metadata size,
and LRU operation complexity — against the baseline on the LIRS workload-a trace.
Only the geometry change (fewer, wider buckets) landed in the Winners; the latency
optimisation hurt hit-rate, and the metadata compaction went unmeasured on the
dimensions it targeted.

## Patterns

- **Wider buckets directly improve hit-rate.** Repartitioning the same total capacity
  into fewer, larger sets gives the per-set LRU a bigger eviction pool, reducing
  premature expulsion of still-live items. The baseline's narrow buckets were a real
  bottleneck on this workload.
- **Strict LRU ordering is load-bearing.** The O(1) swap-to-back optimisation
  preserved the two boundary indices (eviction candidate and MRU) but corrupted the
  recency gradient across intermediate slots, and that degradation was large enough to
  register as a clear hit-rate regression. The O(n) cost buys correctness worth keeping.
- **Unmeasured hypotheses remain open, not disproven.** The timestamp-removal variant
  neither helped nor hurt hit-rate (expected, since ClassicLRU never reads the field),
  but the evaluators for the dimensions it actually targeted — metadata size and
  per-op latency — never ran, leaving the structural benefit unconfirmed.

## Open questions

- Does the bucket-widening gain hold beyond the LIRS workload-a trace, or is it
  specific to its locality pattern?
- Is there a recency-aware O(1) promotion strategy that preserves eviction quality
  better than a plain swap (e.g., a clock approximation or segmented LRU)?
- Can the metadata-size and per-op-time evaluators be enabled so the timestamp-removal
  change gets a real verdict?

## Continuation

The geometry axis is the strongest lead: wider buckets produced a real win, and the
direction has not been saturated — next cycle should test pushing the bucket-width
further or making it adaptive. Drop the O(1) swap angle for ClassicLRU unless a
recency-preserving variant emerges. Before spawning a new metadata-compaction
hypothesis, ensure the necessary evaluators are running so any structural benefit can
actually be confirmed.


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
