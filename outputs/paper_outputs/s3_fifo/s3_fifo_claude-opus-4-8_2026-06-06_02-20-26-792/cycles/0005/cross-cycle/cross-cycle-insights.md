## Recurring wins

- Main-queue *sizing* (the small/main partition ratio) was the single
  most reliable lever for several cycles: enlarging main paid repeatedly
  from 10/90 down through ~4/96, though each step paid less than the last
  until the gain flattened to a sliver and froze.
- Levers that sharpen the *ordering discipline* of the resident main
  queue — making it recency-aware (move-to-front) instead of
  position-blind FIFO — were the only thing to pay anything once sizing
  converged. Reordering *what gets evicted* beats re-admitting or
  protecting weak-signal keys.
- The parent's rotation-on-eviction (refreshing survivor recency) and its
  front-insertion of re-admitted/promoted keys are load-bearing: every
  attempt to "improve" on them confirmed they were earning their keep.

## Recurring failures

- Anything that admits or protects *weak-signal* keys regresses: looser
  promotion gates, deeper ghost history, warm-starting re-admitted
  frequency, tail-insertion of promoted keys. The main queue is
  contention-sensitive — protecting hot residents beats feeding new ones.
- Every form of ghost re-admission gating failed across cycles —
  small-queue routing, throttling, freshness/recency splitting. A ghost
  hit is strong-enough reuse evidence; the direct-to-main path is correct
  and discarding that signal costs more than it saves.
- Distinct-on-paper retention knobs (frequency ceiling, promotion
  threshold) collapse into the partition knob once main is large: they
  buy almost nothing or go fully inert because a thin small-queue evicts
  borderline keys before the gate can act.
- Late-cycle refinements to victim selection (in-place decrement,
  windowed lowest-freq) regressed by discarding the load-bearing
  survivor-refresh rotation.

## Open questions

- Is the ordering axis exhausted after move-to-front, or is there a live
  lever in alternative reorder triggers / recency-aware tail selection?
- With sizing, ghost mechanics, re-admission, and ordering all probed and
  yielding only slivers or negatives, is the configuration converged for
  practical purposes — or is there an untouched mechanism left?
- Is the discriminating signal really accumulated get() frequency rather
  than the ghost hit itself, and does any lever exploit that more directly?

## Per-policy notes

- S3-FIFO: the partition ratio is the dominant but bounded knob, with its
  optimum on this LIRS-style workload at a very thin admission filter
  (~4/96) that tolerates one-hit-wonders without leaking. The
  direct-to-main ghost re-admission path should stay intact. Recency
  structure the partition knob never touched is real but yields only
  marginal gains via main-queue ordering discipline.
