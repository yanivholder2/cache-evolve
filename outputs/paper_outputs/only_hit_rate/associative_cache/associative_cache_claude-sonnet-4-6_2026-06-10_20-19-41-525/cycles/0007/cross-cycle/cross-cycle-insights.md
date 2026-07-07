# Cross-Cycle Insights

## Recurring wins

- **Wider bucket geometry improves hit-rate on workload-a** — repartitioning into
  fewer, larger buckets reduces premature eviction of live items. First confirmed in
  cycle 0001; still contributing as an enabler through cycle 0007. Useful range tops
  out around 32–64 slots; gains beyond that are negligible.
- **Cold-first eviction (`counter == 0` priority) is the strongest single lever
  found.** Shielding any item with at least one hit from scan-traffic displacement
  produced large hit-rate gains (cycle 0002). The `counter == 0` boundary has been
  probed and confirmed optimal for workload-a; raising the threshold regresses.
- **Episodic bulk counter decay prevents warm-lock reliably.** Decrementing all
  counters only when no cold candidate exists fires infrequently but re-enables
  cold-first selection without disturbing the normal eviction path (cycle 0004). Its
  targeted, rare-trigger design is what makes it effective.
- **Positional recency encoded by `remove_at(0)` is load-bearing.** Under mid-point
  insertion, index 0 is the oldest item per bucket, making blind removal-from-front
  a sound recency heuristic. Any mechanism that breaks this implicit ordering (CLOCK,
  post-decay cold scan, selective decay) consistently regresses.
- **The accumulated stack — wide buckets + mid-point insertion + cold-first eviction +
  episodic bulk decay — is a robust local optimum.** It has withstood four consecutive
  cycles of structural challenges (cycles 0004–0007) without being dislodged.

## Recurring failures

- **O(1) LRU swap optimisations corrupt the recency gradient.** Swapping to back
  preserves the two boundary indices but destroys intermediate ordering; the hit-rate
  regression is large enough to be unambiguous (cycle 0001). Do not revisit.
- **Raising the cold threshold above `counter == 0` causes premature hot-set eviction.**
  A single access is a reliable reuse signal on workload-a; requiring two before
  shielding an item consistently regresses (cycle 0003). Threshold tightening is
  closed.
- **Counter-ceiling tuning is inert.** Cap=4 (cycle 0005) and cap=2 (cycle 0007)
  both showed negligible or negative impact. The ceiling magnitude is not a meaningful
  lever at this scale on this workload.
- **LFU warm-fallback after uniform decay is counterproductive.** Bulk decay compresses
  inter-item counter differences to near-zero, so a subsequent minimum-counter scan is
  noisier than the positional recency already encoded in index 0 (cycles 0005–0006).
- **CLOCK and selective decay hurt by eroding active-item protection.** CLOCK's
  per-eviction sweep steadily degrades still-relevant items; selective decay sacrifices
  recency signal for frequency discrimination. Both confirmed regressions (cycle 0006).
- **ARC-style ghost re-admission yields no benefit on workload-a.** Post-eviction
  re-access is too infrequent for ghost tracking to recoup its per-put overhead
  (cycle 0007). Ghost buffers belong on traces with scan-then-reuse patterns.
- **Mid-point insertion combined with LFU warm-fallback is adversarial.** Mid-point
  insertion disrupts the recency structure that LFU warm-selection implicitly depends
  on; the two should not be combined (cycle 0005).
- **Pure geometry pushing is saturated.** Incremental bucket widening beyond the first
  large step returns diminishing gains; the axis was confirmed exhausted by cycle 0004.

## Open questions

- **Per-op latency remains unmeasured across all cycles.** The full accumulated stack
  carries cold-first scan + bulk decay pass + mid-point insertion costs; the simpler
  pure-LFU + wide-geometry path (cycle 0006 v003) approaches the best-known hit-rate
  with fewer mechanisms. A direct latency comparison is the single most deferred and
  actionable audit remaining.
- **Does the accumulated stack's dominance generalise beyond workload-a?** All wins
  and regressions are measured on one trace; the cold-first + positional-recency design
  is tuned to workload-a's scan-dominated locality profile and may not transfer.
- **What is the minimum bucket size below which mid-point insertion ceases to provide
  enough cold candidates to substitute for geometry widening?** Cycle 0003 showed
  mid-point insertion can replace geometry as a cold-pool concentrator, but the
  breakdown point was not probed.
- **Would ARC / LIRS-proper or other two-queue designs — where recency and frequency
  occupy explicit separate queues rather than being co-encoded in position — break
  past the current hit-rate ceiling?** Structurally different families have not been
  tested; every variant so far has worked within the single-queue associative bucket
  model.

## Per-policy notes

- **ClassicLRU (positional, no counters):** Strict O(n) promotion is load-bearing;
  O(1) swap-to-back is confirmed harmful. Wide buckets help but the policy has no
  mechanism to distinguish frequency.
- **Cold-first eviction (counter-gated, positional fallback):** The dominant family
  across all cycles. Best configuration: wide geometry (~32–64 slots), mid-point
  insertion, `counter == 0` eviction priority, episodic bulk decay on cold-pool
  exhaustion, `remove_at(0)` as positional warm fallback. Counter cap is safe but
  inert.
- **LFU / min-counter eviction (frequency-only):** Pure min-counter on wide geometry
  approaches but does not match the cold-first stack. Useful as a simpler alternative
  if latency measurements justify the hit-rate trade-off. Fails when paired with
  uniform bulk decay (signal compression) or mid-point insertion (structure conflict).
- **CLOCK:** Confirmed regression on workload-a. Steady per-eviction aging erodes
  protection of active items faster than episodic bulk decay does; positional recency
  is lost. Not worth revisiting on this workload.
- **ARC-style (ghost re-admission):** Ineffective on workload-a due to low
  post-eviction re-access rate. Mechanism overhead real; benefit zero. Applicable
  only to traces with strong scan-then-reuse locality.
