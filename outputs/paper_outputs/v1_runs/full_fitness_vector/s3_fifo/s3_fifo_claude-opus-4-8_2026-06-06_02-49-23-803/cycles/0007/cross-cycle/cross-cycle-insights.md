## Recurring wins

- **Reallocating capacity into main** (narrowing the small admission
  queue) was the earliest and most reliable hit-rate lever: monotone
  across several cycles but with clearly diminishing returns. The split
  grid is now treated as saturated.
- **Rewarding demonstrated reuse** (raising the freq-saturation ceiling)
  gives small, real hit-rate gains and is orthogonal to the split —
  survival-depth of proven keys is independent of how many fit.
- **Tuning ghost *length* to ~50% of main** was the standout: it banks
  the put-membership-scan latency win while holding or slightly raising
  hit-rate — a rare non-tradeoff. The recoverable near-reuse lives in
  the 25–50% band of recent evictions.
- **Eliminating a scan beats reordering it.** O(1) structural fixes (a
  hash membership index, main-before-small scan order) moved long-stuck
  latency floors; the hash index alone lowered median *and* tail
  together by collapsing resolve, miss, and update paths.
- **Metadata discipline pays off**: every winning change retuned an
  existing field or constant rather than adding scored struct state.

## Recurring failures

- **Loosening the admission/retention filter** consistently regresses:
  first-reuse promotion, non-zero ghost re-admission, and freq=1 seeds
  all hurt hit-rate (often latency too). The baseline's strictness is a
  load-bearing noise filter — re-admitted keys must re-prove from freq=0.
- **Widening the ghost queue** never pays: this workload rewards near
  reuse, not far, so longer reuse-distance detection readmits stale keys
  and costs metadata.
- **Reordering the linear resident scan** instead of removing it
  (front-loading hot keys, a dedicated hot segment, bounded rotation
  loops) repeatedly misses the median target and merely shifts cost into
  the tail.
- **Per-op-time deltas read as noise** unless a change actually alters a
  scan length; most early latency movements were run-to-run jitter.

## Open questions

- Is the workload now jointly hit-rate- and median-latency-bound at this
  capacity, leaving only "serve the same hits more cheaply"?
- Does the winning hash index survive once **unscored container memory**
  (each resident key stored twice) enters scoring?
- Do the structural median wins stay net-positive once tail
  regressions (hash-resize spikes, reordered p99 walks) are weighed?

## Per-policy notes

- All cycles operate on a single **S3-FIFO** baseline. Its three load-
  bearing strictness knobs — two-touch promotion, freq=0 ghost
  re-admission, freq=0 re-proof — should be preserved, not loosened.
- The productive levers form a sequence that has now largely closed:
  small/main split (saturated) → freq-ceiling (saturated) → ghost length
  (knee at 50%, terminal) → ghost content (dual-source main victims, thin
  gain) → scan structure (O(1) index, main-first order).
- Latency on this trace tracks the **ghost membership scan on put** and
  the **resident walk on get**, never the main-queue rotation; rotation-
  bounding schemes cure a pathology the trace lacks.
- The scored `Entry` has held at 24 bytes throughout; the emerging cost
  frontier is unscored container growth from O(1) indexing.
