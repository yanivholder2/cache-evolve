# Cycle 0007 Insight

## Cohort outcome

All three variants probed distinct tuning axes — replacing the warm-lock eviction
heuristic with a post-decay cold scan, adding ARC-style ghost-buffer re-admission,
and halving the counter ceiling — and every one regressed or was negligibly neutral.
v003 was selected as winner purely by elimination, carrying the smallest regression.

## Patterns

- **The positional recency of `remove_at(0)` is already the best warm-lock target.**
  Under mid-point insertion, index 0 is the oldest item in the bucket; swapping to a
  post-decay cold scan risks selecting a more recently inserted item whose counter
  happened to reach zero first, losing the recency signal for no frequency gain. The
  parent's blind index-0 removal was not a design debt — it was the right heuristic
  for this layout.
- **Ghost-buffer re-admission requires workload-a's missing property: frequent
  post-eviction re-access.** The ARC-style mechanism elevated re-admitted keys to
  warm-pool status, but those keys proved no warmer than ordinary new arrivals on this
  trace. The overhead (ghost scan per put, extra eviction on every ghost hit) was
  real; the benefit was not. Re-admission schemes belong on traces with scan-then-reuse
  patterns, not on workload-a.
- **The counter ceiling axis is conclusively exhausted.** Cap=4 was confirmed inert in
  cycle 0005; cap=2 now produces a marginal regression without any compensating gain.
  Neither the ceiling magnitude nor its existence is a meaningful lever on this workload
  at this cache size.
- **Four consecutive cycles of structural and parametric challenges have failed to
  dislodge the accumulated-stack local optimum.** The pattern is no longer ambiguous:
  the complementarity of cold-first eviction, positional recency, mid-point insertion,
  and episodic bulk decay is robust on workload-a.

## Open questions

- The per-op latency comparison recommended since cycle 0005 remains unmeasured. Does
  the simpler pure-LFU + wide-geometry path (cycle 0006 v003) offer a competitive
  efficiency trade-off despite its hit-rate deficit?
- Would ARC-style ghost re-admission perform differently on a trace with stronger
  post-eviction temporal locality, or is the mechanism structurally mismatched to the
  cold-first eviction family?

## Continuation

The current mechanism family is saturated on hit-rate for workload-a; further
eviction-policy tuning is unlikely to yield gains. The single most actionable next
step is the latency audit deferred across multiple cycles: measure per-op cost for the
full accumulated stack versus the simpler pure-LFU + wide-geometry alternative to
determine whether the hit-rate delta justifies the mechanism complexity. If latency
parity is found, the simpler path should become the new baseline and free up design
space for structural alternatives. Do not revisit ghost re-admission, post-decay cold
scans, or any further counter-ceiling variation — all three axes are now confirmed
closed on this workload.
