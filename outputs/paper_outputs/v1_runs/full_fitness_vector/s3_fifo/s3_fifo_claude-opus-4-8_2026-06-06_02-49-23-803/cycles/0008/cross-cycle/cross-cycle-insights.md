# Cross-Cycle Insights

## Recurring wins

- Reallocating capacity into the main queue (narrowing the small/main
  split) was the first and most durable hit-rate lever — monotone for
  several cycles until it flattened into diminishing returns.
- Rewarding *demonstrated* reuse (taller freq ceiling) compounds with the
  split and is orthogonal to admission — it deepens durability only for
  keys that already earned it.
- The ghost has a confirmed length knee at ~50% of main: it banks the
  put-scan latency win and even recovers hit-rate, a rare non-tradeoff.
- Eliminating hot-path linear scans structurally (O(1) hash index for get
  in 0007, O(1) ghost membership for put in 0008) beats reordering them —
  these moved long-frozen p50 floors and often p99 too.
- Metadata discipline pays off later: the 0007 index made the per-entry
  `key` field dead weight, deleted in 0008 to break a seven-cycle
  metadata-size plateau.

## Recurring failures

- Loosening retention/admission filters (first-reuse promotion, non-zero
  or freq=1 ghost re-admission) repeatedly hurt hit-rate: the baseline's
  strict freq=0 re-proving is load-bearing noise filtering, not over-strict.
- Widening the ghost queue never helps — this workload rewards near reuse,
  not far reuse — and adding ghost-sizing struct state costs metadata.
- Reordering a linear scan to make the median hit shallow (front-loading
  hot keys, dedicated hot segment, bounded rotation loop) consistently
  misses its target and merely shifts cost to the tail.
- Stacking levers assumed additive often interferes (split × ghost),
  though split × freq-ceiling proved genuinely orthogonal.

## Open questions

- Now that both hot-path scans are O(1), is there any latency or memory
  lever left, or is the workload at a joint hit-rate / median-latency /
  scored-size floor?
- Which O(1)-ghost design wins once *unscored container memory* is weighed
  — the duplicate-key HashSet or the tombstone-folded single map?
- Do tombstone / dual-table designs accumulate stale entries that degrade
  under longer or different inflow than tested?
- Are more dead per-entry fields now deletable thanks to the hash index?

## Per-policy notes

- All cycles tune a single S3-FIFO variant; hit-rate has been bit-identical
  for many cycles, treated as closed and capacity-bound at this size.
- The policy's strictness (two-touch promotion, freq=0 ghost re-admission)
  is protective on this near-reuse workload — retune existing constants
  rather than relax filters or add state.
- Latency tracks specific scans, not run-to-run jitter: per-op deltas are
  noise unless a change alters an actual scan length or structure.
- The frontier has migrated from hit-rate → ghost length/content →
  structural scan elimination → unscored container/tombstone memory, which
  is now the expected next binding constraint.
