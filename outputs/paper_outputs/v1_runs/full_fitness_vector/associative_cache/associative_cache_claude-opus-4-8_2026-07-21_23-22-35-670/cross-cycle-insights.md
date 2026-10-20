## Recurring wins

- Trimming dead metadata off the eval path is cheap, monotonic, and
  behaviourally inert on hit-rate: fields carried only by off-eval-path
  policies peel off repeatedly with no accuracy cost, driving `Item` down
  to and holding it at the metadata floor.
- Keep scan-resistance state *off* the measured `Item` — in the bucket
  struct, then a parallel vector, then a scalar mask — so hit-rate and
  metadata-size stopped being antagonistic; the size metric weighs only
  `Item`, so bucket-side state is free on that axis.
- The index-preserving splice is the portable latency fix: mirroring a
  policy's own single-element relocation onto the parallel reuse state
  (vector, then `u64` mask) instead of key-searching/rebuilding each hit
  collapses p99 tails while holding hit-rate bit-for-bit.
- Genuine two-segment separation (SLRU/2Q protected segment) is the one
  lever that ever broke the front-admission hit-rate ceiling — structure,
  not promotion timing, unlocked headroom.
- Orthogonal levers compose cleanly: independent hit-rate (hash spread)
  and latency (reuse-set representation) mechanisms spliced into a strict
  Pareto improvement with no interaction cost.

## Recurring failures

- Anything that widens buckets or turns O(1) eviction into a linear scan
  taxes every op; plain associativity increases buy no hit-rate at this
  working-set size.
- The geometry seam is falsified on both sides — wider buckets coarsen
  hash spread and narrower buckets raise conflict misses; the default
  set/slot split sits at the optimum.
- Weakening permanent one-hit protection always regresses: second-chance
  aging, CLOCK-style clearing, two-hit admission gates, and clearing the
  mask at saturation all shed immunity from genuinely hot keys.
- Frequency/LFU victim selection loses to recency-front every time it is
  tried.
- Re-timing promotion on a single short list (single-step, reuse-bit
  gating) does not buy hit-rate — small buckets lack the slots for
  gradual promotion.
- Optimizing sub-resolution fixed per-op costs (SipHash→integer hash,
  below-cap single-element rotation) is correct but unrewardable: only
  heap-vector paths clear the ~100ns timer.
- Overlay hygiene: single-file variants that don't inherit the full
  parent layout silently revert prior wins (the 16-byte `Item`) — a loss
  by omission, recurring until anchored-parent discipline was enforced.

## Open questions

- With hit-rate and latency now co-located at their floors, is any
  harness-resolvable dimension left, or is the incumbent at the joint
  frontier for this trace?
- Are the remaining tails purely the policy's own `items` front-shifts
  (`insert(0)`/`remove(0)`), and is there any resolvable gain there?
- Is two-segment SLRU at diverse small buckets the genuine structural
  hit-rate ceiling, or does headroom exist beyond same-signal re-encodings?

## Per-policy notes

- LRU / front-admission: the front slot is the settled admission-geometry
  optimum; a fresh unproven miss should be the next eviction. Midpoint /
  wider-probation insertion is strictly worse.
- Reuse-bit protection: permanent binary one-hit protection fits this
  trace and sits at/above the sweet spot; explicit protection is redundant
  with front-admission under move-to-back-on-hit and only adds value over
  a policy that does not positionally segregate reused items.
- SLRU / segmented: the ceiling-breaking incumbent, delivered at the
  metadata floor; the protected-cap knob is mapped end-to-end and reads
  nearly flat (both extremes falsified, mid-range trivial). Per-access
  protected rotation is overhead except in the saturated all-protected
  bucket, where it silently maintains intra-protected eviction order.
- LFU / frequency: consistently a worse victim signal than recency; drop.
