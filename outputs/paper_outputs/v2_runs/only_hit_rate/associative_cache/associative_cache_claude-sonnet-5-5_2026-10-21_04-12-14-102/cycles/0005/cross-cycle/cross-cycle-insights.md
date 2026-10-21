## Recurring wins

- Structural separation of one-touch entries from re-referenced ones (a probationary/protected split inside each bucket) is the one lever that has paid off clearly, and it has held across cycles 2-5.
- A small probation share (about slots/16, roughly 4 slots in a wide bucket) beats larger shares. Independent routes converged on it.
- A lean positional split, with no counters, ghost state or adaptive targets, captures nearly all of the gain over plain LRU. It was rebuilt on the baseline twice and won both times.
- Wider buckets gave a small, real hit-rate gain (cycle 1) and are the base geometry.

## Recurring failures

- Weakening hit-path promotion in plain LRU buckets (transposition, midpoint, half-promotion) lost hit rate.
- Delaying promotion (two-hit rules) hurt, because small probation segments evict entries before they get a second hit. First-hit promotion looks hard to beat.
- Ghost history (rings, ghost-hit direct promotion, adaptive targets) added nothing over the split. A ghost hit is weak reuse evidence here.
- Eviction-side tweaks do not help: small-window frequency victim choice was flat, and recency-stamp overrides of probation-first eviction lost hit rate.
- Placement that moves keys outside their hashed bucket fails the slot-legality invariant.
- Shrinking probation below the slots/16 size slightly hurt, so size tuning is exhausted.
- Per-op-time and metadata-size have been missing from the fitness vector in every cycle. All claimed wins are hit-rate only.
- Several variants are compared with the baseline rather than their parent, so their gains are not increments.

## Open questions

- Do the hit-rate gains survive once per-op-time and metadata-size are measured? In particular, what do reordering and demotion on hits cost, and does the lean split really match baseline footprint?
- Is there any hit-rate headroom left in protected-segment handling or promotion, or is the design at its ceiling on this trace?
- Is the tuning specific to the LIRS-style trace? Robustness on other workloads is untested.
- Could a stricter ghost rule (ghost hit plus a second signal) rescue ghost history? This is low priority.
- Is an invariant relaxation worth considering to allow multi-bucket placement?

## Per-policy notes

- Plain move-to-back LRU: the baseline. It is hard to beat via promotion tweaks, and its misses are mostly policy misses, not hash imbalance.
- Segmented LRU (probation/protected) with a small positional probation: the current best. Use the lean version as the base.
- Segmented LRU with extras (ghost rings, per-entry counters, adaptive targets, stamps): no gain over the lean split, with unpriced state cost. Candidates for removal.
- Frequency-aware victim choice: flat in a small window.
- Two-choice placement: rejected at the invariant gate.
