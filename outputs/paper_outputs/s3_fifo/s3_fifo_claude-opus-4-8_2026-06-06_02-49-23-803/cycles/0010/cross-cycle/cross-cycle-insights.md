## Recurring wins

- **Reallocate capacity into main** (narrow the small/admission filter):
  the single most reliable hit-rate lever for early cycles. Monotone but
  diminishing; flattened to a knee just past the narrowest split tried.
- **Reward demonstrated reuse, never loosen the gate**: raising the
  freq/retention ceiling for already-proven keys gives small, repeatable
  hit-rate gains orthogonal to capacity, and finally broke the long
  hit-rate plateau in a late cycle. Confirmed across many cycles that hot
  keys had headroom under a too-low ceiling.
- **Ghost length has a knee at ~50% of main** — a rare strict-dominating
  point (faster put *and* equal-or-better hit-rate). Recoverable near
  reuse lives in the 25–50% band of recent evictions.
- **Eliminate linear scans structurally, don't tune them**: O(1) hash
  index for the resident (get) lookup and O(1) ghost-membership probe for
  put each moved long-frozen latency floors, sometimes median+tail
  together. This was the dominant late-cycle source of wins.
- **Metadata via field deletion/relocation**: removing the redundant `key`
  field (made dead by the hash index) and relocating freq/value off
  `Entry` drove scored size down repeatedly with no hit-rate/latency cost.

## Recurring failures

- **Loosening the admission/retention filter for weakly-touched items**
  consistently regresses (first-reuse promotion, non-zero ghost
  re-admission, freq=1 seed on re-admit). The baseline strictness is
  load-bearing noise-filtering; re-admitted keys must re-prove from freq=0.
- **Widening the ghost queue** never pays off: this trace rewards near
  reuse, not far reuse, so wider windows readmit stale keys and add scan
  cost.
- **Reordering a linear scan instead of removing it** (front-loading hot
  keys, dedicated hot segment, scan reorder) misses the median and merely
  shifts cost to the tail. get-p50 is structural, not a hot-key-scatter
  problem.
- **Adding struct state for a tuning knob** (ghost-size field) — no benefit,
  grows metadata.
- **Curing pathologies the workload lacks**: bounded second-chance rotation
  bought no tail win (put tail isn't rotation-driven), and eliding the
  saturated freq store added branch-misprediction tail jitter for no
  median gain.

## Open questions

- The **unscored container-memory frontier** (separate ghost set vs.
  folded-tombstone single map) has been flagged ~four cycles running but
  never directly contested — these designs make opposite memory tradeoffs
  for identical scored results.
- Scored metadata now reports 0 from a zero-sized `Entry`; should the
  metric be redefined to capture real per-slot footprint (the map tuple)
  rather than relocatable type size?
- Where is the retention-ceiling optimum before it starts holding
  gone-cold keys and regresses?
- With hit-rate, both hot-path scans, scored entry size, and median
  latency all at floors, is any scored axis left with genuine headroom, or
  is the cost the scoring captures exhausted?

## Per-policy notes

- All cycles operate on a single policy: **S3-FIFO** on workload-a.
- The workload is **hit-rate-bound at this capacity** for most of its
  history; late gains come from serving the same hits more cheaply, not
  from more hits.
- Reuse on this trace is **near, not far**: narrow ghost windows and strict
  admission win; long-reuse-distance detection is penalized.
- Latency decomposes into distinct scans (ghost membership on put, resident
  walk on get); each fell to a structural O(1) fix rather than length
  tuning. Median wins often redistribute cost to the tail.
- Per-op-time should be scored only when a change alters an actual scan
  length; otherwise it reads as run-to-run noise.
