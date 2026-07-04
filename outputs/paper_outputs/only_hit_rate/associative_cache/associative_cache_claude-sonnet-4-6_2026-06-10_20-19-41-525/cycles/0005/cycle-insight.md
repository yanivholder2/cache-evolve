# Cycle 0005 Insight

## Cohort outcome

The cohort tested three refinements of the cycle 0004 decay-based winner: replacing
positional-LRU fallback with LFU minimum-counter selection after decay, stacking
mid-point insertion onto the narrow-geometry LFU-fallback configuration, and capping
counters at a bounded maximum. All three failed to improve on their parents — two
regressed and one was neutral — making the counter-cap variant (v003) the winner by
elimination rather than by merit.

## Patterns

- **Uniform counter decay destroys the frequency signal it was meant to preserve.**
  After a single pass that decrements every counter by the same amount, inter-item
  counter differences compress to near-zero; a minimum-counter scan then becomes
  noisier than the recency order already encoded in bucket position. The positional
  `remove_at(0)` fallback was already well-aligned with insertion ordering and
  outperforms frequency selection in this compressed regime.
- **Mid-point insertion and LFU warm-fallback are adversarial, not additive.**
  Cold-first eviction scans by counter value and is position-agnostic, so mid-point
  insertion's benefit — keeping new items away from the eviction front — is irrelevant
  to it. Meanwhile, the LFU warm-fallback still implicitly relies on the
  promotion-to-back recency structure; disrupting that structure with midpoint inserts
  costs more than the reduced cold-pool exhaustion saves. The two mechanisms should
  not be combined on narrow geometry.
- **The counter-cap hypothesis was sound in theory but inert on this workload.**
  Unbounded accumulation does not manifest as a measurable hit-rate problem on
  workload-a at the tested scale; the decay mechanism already fires frequently enough
  to prevent permanent warm-lock from high-counter items. The cap is safe to carry
  forward as defensive hardening but opens no new lever.
- **The wide-bucket + mid-point + cold-first + decay configuration is a local optimum
  for hit-rate on workload-a.** Two consecutive cycles of refinement attempts have
  returned regressions or neutrality; the design space of eviction-policy tuning on
  this geometry and trace appears saturated.

## Open questions

- Is hit-rate on workload-a genuinely saturated, or do structurally different eviction
  families (CLOCK, ARC-style adaptive replacement, LIRS-proper) offer a path past the
  current ceiling?
- Per-op latency has not been measured for the accumulated mechanism stack; is the
  combined cost of cold-first scan + decay pass + mid-point insertion acceptable, and
  does narrow geometry + LFU fallback offer a better hit-rate/latency trade-off?
- Do the current regressions generalise — i.e., would LFU-after-decay and mid-point +
  LFU also fail on traces with different locality profiles, or are these failures
  specific to workload-a's scan-dominated pattern?

## Continuation

Hit-rate improvements on workload-a appear exhausted for the current mechanism family.
Next cycle should pivot on one of two axes: (a) measure per-op latency across the
full accumulated stack to determine whether the narrow-geometry or wide-geometry path
offers a better efficiency trade-off — this is the last unprobed dimension with a
clear comparison point — or (b) introduce a structurally different eviction policy
(e.g., CLOCK or an adaptive two-queue scheme) to break out of the local optimum. Do
not revisit LFU-after-decay fallback or mid-point + LFU combinations — both are
confirmed counter-productive. The counter cap (v003) is confirmed safe and should
be inherited silently as part of the baseline.
