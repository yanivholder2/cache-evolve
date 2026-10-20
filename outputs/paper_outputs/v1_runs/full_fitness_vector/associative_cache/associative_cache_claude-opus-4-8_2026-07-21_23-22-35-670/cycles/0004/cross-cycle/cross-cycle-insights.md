## Recurring wins

- Trimming dead metadata off the eval path is the most reliable move: each
  cycle that peeled a field carried only by off-eval-path policies gained
  size with zero hit-rate cost. The trims compose additively and stay
  monotonic field after field.
- Parking state off the measured `Item` (in the bucket struct) is free on
  the size metric, which dissolved the early hit-rate-vs-size antagonism —
  scan resistance became reachable while holding the metadata floor.
- Stateless probationary-front insertion is the cheap incumbent: it lifts
  hit-rate at the floor with clean tails and no per-item state, and it
  composes cleanly with slimmed layouts.
- Permanent binary reuse-protection consistently fits this trace; the proven
  disjoint mechanisms (a reuse signal + layout slimming) stack without
  interaction because they touch disjoint code.

## Recurring failures

- Anything that lengthens a linear scan (wider buckets, scan-based eviction
  picks) taxes every operation — scan cost is the dominant hidden price, so
  hit-rate bought this way is never free.
- Frequency/LFU victim selection loses to recency-front every time it is
  tried; recency is the better victim signal on this trace.
- Weakening permanent protection (second-chance / CLOCK-style aging, wider
  midpoint probation) sheds immunity too fast and collapses toward plain LRU.
- Re-encoding the same scan-resistance signal (explicit protection layered
  onto front-admission) yields no gain — under move-to-back-on-hit the front
  slot is already the probationary one, so the levers are redundant.

## Open questions

- Where does the remaining hit-rate headroom live now that stateless and
  stateful paths all converge on the same ceiling — does exceeding it require
  changing the base policy's ordering rather than layering state on top?
- Recurring metadata regressions that a variant's own diff cannot explain
  keep appearing (overlay/inheritance artifacts?). How to stop single-file
  variants from silently forfeiting prior-cycle wins in files they don't touch?
- The persistent upward p99 tail drift under flat p50 on metadata cuts —
  structural, or measurement noise?
- Is the metadata floor near its true bottom, or can it shrink further before
  colliding with a field the eval path actually reads?

## Per-policy notes

- LRU / recency-front: the settled winner. Front slot is the optimum on the
  admission-geometry axis; a fresh unproven miss should be the next evicted.
- Reuse-protection (binary, permanent): additive with layout slimming; must
  use an index-preserving swap to relocate reuse bits (key-matched rebuild is
  quadratic and blows out tails). Adds value only over a policy that does not
  already positionally segregate reused items.
- Second-chance / CLOCK aging: rejected — sheds protection too fast.
- Frequency / LFU: rejected as a victim signal — loses to recency-front.
