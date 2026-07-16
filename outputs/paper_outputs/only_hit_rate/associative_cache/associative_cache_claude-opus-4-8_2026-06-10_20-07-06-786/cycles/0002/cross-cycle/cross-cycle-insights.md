## Recurring wins

- Demanding proof of reuse before granting cache protection keeps winning:
  frequency-first victim selection (C1) and segmented promote-on-second-hit
  (C2) both beat their parents by shielding the reusable working set from
  one-hit-wonders and scan traffic.
- Repurposing the dormant `lfu_counter` as a live signal — a hit count (C1)
  then a promote flag (C2) — has produced the cohort Winner two cycles
  running.
- High-leverage changes stay intra-bucket: victim/admission logic moves the
  needle while resident-set structure, routing, capacity invariants, and the
  policy enum / JNI mapping stay untouched.

## Recurring failures

- The orthogonal levers do not compose. Stacking probationary front
  admission onto frequency-first eviction (C2 v001) makes a fresh key both
  recency-oldest and min-frequency, so it is evicted immediately — a direct
  conflict, not an additive gain.
- Tuning a single frequency scalar underperforms explicit structure.
  Counter aging (C2 v002) regressed by stripping the durable-count
  protection that makes hot keys valuable.
- Geometry coarsening / wider buckets (C1 v001) is collision-bound and gave
  the smallest gain — diminishing returns, dropped.

## Open questions

- Can the O(bucket_size) linear victim/probationary scan be made O(1) (e.g.
  a tracked segment boundary) without losing the hit-rate gain? Its per-op
  cost has been structurally present but unmeasured every cycle.
- Fitness vectors report only hit-rate; the cost side (per-op time, scan
  cost) of each trade remains untracked.
- Workload appears firmly capacity-collision-bound, not staleness-bound
  (immortality helped, aging hurt). Is there any shifting-working-set regime
  where staleness would dominate?
- Is there a probationary/protected sizing ratio that beats the implicit
  split, or does promote-on-second-hit already capture most of it?

## Per-policy notes

- LRU (baseline): pure-recency victim choice is the thing both winners
  displace; weak against scan traffic and one-hit-wonders.
- LFU / frequency-first: strong victim rule (C1 Winner); "once-hot
  immortality" is a feature on this trace, so do not age counters. Still
  pays a linear min-frequency scan.
- Segmented-LRU (promote-on-second-hit): current strongest baseline (C2
  Winner) and the best scan-resistant admission scheme so far; build cost
  refinements on top of it.
