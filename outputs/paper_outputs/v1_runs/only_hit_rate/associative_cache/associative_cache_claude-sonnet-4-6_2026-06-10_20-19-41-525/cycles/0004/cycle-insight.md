# Cycle 0004 Insight

## Cohort outcome

All three variants converged on the same root problem — what happens when the cold
pool runs dry — and each attacked it from a distinct angle: restoring cold candidates
via counter decay (winner v001), improving warm-fallback selection quality via LFU
(v002), and enlarging the cold pool via modest geometry widening (v003). Only the
counter-decay variant was selected, but all three showed consistent hit-rate gains,
confirming that warm-lock is a real, non-trivial steady-state failure mode for the
cold-first eviction family.

## Patterns

- **Counter decay is an effective self-correcting patch for warm-lock.** Decrementing
  all counters by 1 only when no cold candidate exists fires rarely but reliably,
  re-enabling cold-first selection without touching the normal eviction path. Its win
  confirms that cold pool exhaustion does degrade hit-rate in steady state and that a
  targeted, infrequent intervention is sufficient to fix it.
- **LFU warm-fallback is independently valuable.** On narrow geometry — where cold
  pool exhaustion is routine — replacing the LRU-position fallback with minimum-counter
  selection produced a substantial hit-rate gain with no geometry change. This is a
  clean orthogonal axis: it improves eviction quality during the warm phase without
  depending on bucket width or insertion site.
- **Mid-point + cold-first geometry gains saturate at ~32 slots per bucket.** Adding a
  factor-2 widening to the narrow-geometry mid-point path matched or marginally exceeded
  the wide-bucket (factor-4) configuration from cycle 0003, confirming that further
  geometry pushing on mid-point paths is exhausted. The useful geometry range for
  mid-point insertion ends around 32 slots.

## Open questions

- Can counter-decay (v001) and LFU warm-fallback (v002) be stacked productively? They
  are aimed at different aspects of warm-lock — restoring cold candidates versus
  choosing better among warm ones — and may compound rather than overlap.
- Does the counter-decay mechanism remain stable over very long traces, or does periodic
  bulk decrement cause oscillation in counter values that destabilises the cold/warm
  boundary?
- Would the LFU fallback add further gain to v001's wide-bucket + mid-point + decay
  configuration, or is decay sufficient to keep the cold pool from exhausting on wide
  geometry?

## Continuation

The most promising next step is combining v001's counter-decay with v002's LFU fallback:
the two mechanisms are orthogonal in function (one restores cold supply, the other
improves warm selection quality) and each was tested independently without the other.
Drop pure geometry widening — that axis is now doubly confirmed saturated for mid-point
insertion paths. Before adding further mechanisms, run a per-op latency pass on v001 to
verify that the decay scan does not impose a meaningful overhead on the normal eviction
path.
