## Recurring wins

- Reallocating capacity toward the main queue (shrinking the small
  admission queue) has been the only move to lift hit-rate so far —
  giving the main queue more room beats giving individual entries more
  protection.

## Recurring failures

- Loosening retention to make main "stickier" (first-reuse promotion,
  re-admitting ghosts at non-zero frequency) has consistently hurt
  hit-rate; the baseline's strict two-touch / freq=0 filtering acts as a
  load-bearing noise filter against one-hit-wonders.

## Open questions

- How far can the small/main split be pushed before the small queue is
  too narrow to screen one-hit-wonders?
- Are admission-side and retention-side knobs orthogonal, or does
  resizing the small queue change the value of the promotion threshold?

## Per-policy notes

- S3-FIFO: capacity-allocation knobs (small/main split, ghost sizing)
  are where hit-rate actually moves; pair split changes with the
  unchanged strict thresholds rather than loosening them. Treat
  per-op-time deltas as measurement noise unless a change touches the
  hot path or the scanned resident-set size.
