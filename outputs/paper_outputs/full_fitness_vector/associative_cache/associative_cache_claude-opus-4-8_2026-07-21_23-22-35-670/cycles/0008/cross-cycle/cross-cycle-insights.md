## Recurring wins

- **Trim dead metadata off the eval path.** Fields carried only by
  off-eval-path policies are pure dead weight; peeling them one at a time
  is monotonic and hit-rate-inert, and it reclaims the metadata floor
  cheaply. This paid off repeatedly and was the first unambiguous move.
- **Park state off the measured `Item`.** The size metric weighs only
  `Item`, so scan-resistance state kept in the bucket struct (reuse bits,
  segment boundaries, caps) is free on the size axis. This exploit broke
  the early hit-rate/metadata antagonism and has held every time.
- **Index-preserving relocation of parallel state.** Mirroring a policy's
  own single-element move onto its parallel reuse vector — instead of
  key-matched snapshot-and-rebuild each hit — repeatedly collapsed blown-out
  p99 tails while holding hit-rate bit-for-bit. It is the portable latency
  win of the run.
- **Segmentation (SLRU/2Q-style) is the one lever that raised hit-rate.**
  Reserving a protected segment for twice-hit keys was the only change to
  cross the front-admission ceiling, and it does so at the metadata floor.

## Recurring failures

- **Re-timing promotion on a single list.** Slow/gradual promotion
  under-protects hot keys on short buckets; reuse-bit gating is redundant
  with position. Gradual promotion needs more slots than a small bucket has.
- **Frequency/LFU victim signals.** LFU tiebreaks and least-frequently-reused
  eviction lost to plain recency-front every time it was tried.
- **Second-chance / CLOCK aging.** Self-clearing protection sheds immunity
  too fast and collapses toward LRU; permanent binary protection fits this
  trace.
- **Stacking explicit protection onto front-admission.** Under
  move-to-back-on-hit the front slot is already the probationary one, so an
  added reuse bit is behaviorally redundant.
- **Buying hit-rate through more depth/memory/protection.** Wider
  associativity (coarsens hash spread), 2Q ghost memory (over-inflates the
  protected set), and near-total protection caps all regressed — three walls
  mapped around the segmentation ceiling.
- **Sub-resolution micro-optimizations.** Removing a single-element rotation
  below the ~100ns timer is correct but unrewardable by the harness.
- **Overlay hygiene by omission.** Single-file variants repeatedly reverted
  the parent's slim `Item` back to the base layout — a win lost by omission,
  not logic. Recurred across several cycles until parents were anchored
  explicitly.

## Open questions

- Is two-segment SLRU at diverse small buckets the structural hit-rate
  ceiling for this trace, or is there headroom no explored lever reaches?
- Where does the protected cap actually optimize? Both extremes are
  falsified and mid-range reads nearly flat — treat as settled or search
  the mid-range once more?
- With the O(n²) reuse rebuild gone, do remaining tails live in the
  `insert(0)`/`remove(0)` front shifts, or is the path at its latency floor?
- Has the binding constraint shifted entirely from hit-rate to
  harness-resolvable latency?

## Per-policy notes

- **ClassicLRU / recency-front:** the settled baseline. Recency-front victim
  choice beat every frequency-based alternative; the front slot is the
  optimum on the admission-geometry axis (midpoint/wider probation regress).
- **Reuse-protection (binary):** permanent protection is the feature, not a
  bug, on this trace; refinements must preserve it. Effective only over a
  policy that does *not* already positionally segregate reused items.
- **SLRU / segmented:** the incumbent and the only ceiling-breaker; delivers
  its gain at the 16-byte floor. Cost moved from metadata to per-access
  rotation latency. Protected-cap knob mapped end-to-end and is nearly flat;
  per-hit protected rotation is overhead except in the saturated
  (all-protected) bucket, where it silently maintains eviction order.
- **2Q / ghost memory:** cross-eviction memory did not lift hit-rate — a
  re-reference is not proof of long-term heat, and blanket pre-protection
  erodes reuse-bit discrimination while taxing every put.
