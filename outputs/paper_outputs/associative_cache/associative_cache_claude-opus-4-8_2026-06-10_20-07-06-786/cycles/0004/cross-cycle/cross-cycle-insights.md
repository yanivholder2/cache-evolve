# Cross-Cycle Insights

## Recurring wins

- **Earned, durable protection beats raw recency.** Every hit-rate gain has
  come from segmenting the bucket and shielding a proven working set from
  one-hit-wonders and scan traffic — promote-on-(first/second)-hit plus a
  load-bearing protected segment.
- **Probation order is high-leverage.** Giving a fresh key a full lap to earn
  promotion (oldest-first / FIFO probation) recovers rising keys that
  newest-first churn discards — a clean hit-rate lift with no metadata change.
- **Victim/admission logic is the productive surface.** The strongest wins
  stayed intra-bucket (victim choice, probation order, segmentation); resident
  set, routing, capacity invariants, and the policy enum were never touched.

## Recurring failures

- **Anything that erodes earned protection regresses.** Counter aging,
  front/probationary admission composition, raising the promotion threshold,
  capping/demoting the protected segment — all failed for the same reason on
  this collision-bound trace: they make protection harder to earn or easier to
  lose.
- **Levers do not freely compose.** Stacking front admission onto
  frequency-first eviction made fresh keys simultaneously oldest and
  min-frequency, i.e. instant victims — orthogonal in isolation, conflicting
  when combined.
- **Approximating away granularity costs hit-rate.** Coarse one-bit (CLOCK)
  recency underperforms full positional ranking in small buckets; geometry
  coarsening is collision-bound with diminishing returns.

## Open questions

- Is the FIFO-probation + promote-on-first-hit scheme now at its hit-rate
  ceiling, leaving only cost-side wins (banking reorder-free tiebreak savings)?
- Does *looser* (larger, uncapped) protected sizing help, since tightening it
  consistently hurts?
- The O(bucket_size) linear victim/probation scan persists across all variants
  but is largely unmeasured — what is its true per-op cost?
- Is there any workload regime (shifting working set) where staleness would
  dominate, or is this trace firmly collision-bound?

## Per-policy notes

- **Pure LRU (baseline):** victim/admission logic is the weak point; recency
  alone under-protects the reusable working set against scans.
- **Frequency-first / LFU-counter:** strong initial win by repurposing the
  dormant counter as a live hit count; "once-hot immortality" is a feature
  here, not a bug — aging it removed the protection that made it valuable.
- **Segmented-LRU:** the durable winner. Explicit segmentation with
  promote-on-reuse beat single-scalar frequency tuning. FIFO (oldest-first)
  probation is the current best; one probationary hit is sufficient proof of
  reuse; the protected cap/demotion branch is effectively dead code (segment
  never saturates).
- **CLOCK / reference-bit:** regresses — too coarse to replace positional
  recency in small buckets.
