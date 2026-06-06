# Cross-Cycle Insights

## Recurring wins

- Giving the main resident queue more capacity (raising the small/main
  partition ratio toward the policy default) is so far the only net hit-rate
  improvement, though the gain is modest and likely bounded.

## Recurring failures

- Admitting more keys into the main queue keeps backfiring: both looser
  promotion thresholds and deeper ghost history (via direct-to-main
  re-admission) regressed. The main queue is contention-sensitive, so
  protecting hot residents beats feeding new candidates.
- "Move toward the paper's defaults" is not uniformly safe — the sizing
  default helped, but the promote-on-any-reuse threshold and a wider history
  window both hurt on this workload.

## Open questions

- Where is the optimum for the small/main ratio? The winning change was small;
  pushing past ~10/90 may sharpen the gain or start leaking one-hit-wonders
  through a shrinking admission filter.
- Could deeper ghost history pay off if ghost hits were gated (re-admit via the
  small queue, or only when fresh) rather than pushed straight to main? History
  depth and the re-admission rule appear coupled.

## Per-policy notes

- S3-FIFO: stricter admission filtering is load-bearing on the observed LIRS
  workload, not an oversight. The partition ratio is a real but bounded lever;
  threshold-loosening and naive ghost-deepening both crowd the main queue and
  cost more than they save.
