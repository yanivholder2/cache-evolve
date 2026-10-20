# Cycle 0007 Insight

## Cohort outcome

This cycle probed the headroom around the new 7/8 protected-cap champion
from three angles: a larger geometry step to 15/16 (v001), a stricter
two-hit probation gate (v002), and a last-key get cache to skip the
HashMap probe (v003). Both surviving Variants held the 24-byte node but
regressed their bases — neither advanced the frontier. The lone latency
probe (v003) never produced a measurement, lost to a simulator infra
failure rather than on merit.

## Patterns

- The protected-cap headroom between 7/8 and the starvation cliff is now
  closed from the top. Cycle 0006 found free hit-rate at 7/8; stepping to
  15/16 confirmed a sharp starvation cliff in that gap — the near-full
  protected segment leaves the fast path no room, so get/put p50 and p99
  all jumped for a small hit-rate gain. 7/8 sits just below the breakdown
  point, and there is little usable geometry left above it.
- Admission cleverness keeps failing for the same workload reason. As in
  Cycle 0005's two-hit gate, the two-hit warm filter drains short-reuse
  keys out of probation before their second hit lands, so it discards
  genuine repeaters and drops hit-rate — while adding per-hit bookkeeping
  that nudges get p50 up. The free-bit mechanism worked; the policy is
  the recurring loser on this short-reuse trace.
- The mechanism-vs-policy split holds yet again: 15/16 sizing and the
  warm-bit packing both kept the 24-byte layout byte-for-byte; every loss
  came from behavior (geometry starvation, admission policy), not from the
  representation changes.

## Open questions

- With both the top of the geometry headroom and admission gating now
  exhausted, is the 7/8 champion effectively at this trace's
  hit-rate/latency frontier?
- Does the H03 last-key cache — the only attempt that actually targeted
  the named get hot-path cost (HashMap hash+probe) — have merit? It was
  never evaluated, so the latency-via-traversal/hashing thread remains
  genuinely untested, not refuted.

## Continuation

Keep Cycle 0006's 7/8 protected cap as the standing base; nothing this
cycle displaced it. Drop the 15/16 geometry step (starvation cliff
confirmed) and the two-hit warm gate (admission gating is tried-and-
regressed on this short-reuse workload — stop revisiting it). The one
live thread worth re-running is H03's last-key get cache: it attacks the
real hot-path cost and died on infra, not measurement, so re-issue it
before concluding the champion is at the frontier. If pursuing latency
further, stay on actual hot-path costs (hashing, link surgery, cache-line
traversal), not geometry or admission policy.
