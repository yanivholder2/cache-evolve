# Cross-Cycle Insights

## Recurring wins

- Explicit segmentation with a single-hit promotion (promote-on-first-reuse,
  straight to MRU) is the strongest scan-resistant scheme found; it beat pure
  LFU victim selection and plain single-position LRU when first introduced and
  has survived every subsequent challenge.
- FIFO (oldest-first) probation outperforms newest-first churn: giving a fresh
  key a full lap to earn promotion recovers rising keys and lifted hit-rate
  with no metadata change.
- "Once-hot immortality" — durable protection that is hard to lose — is
  repeatedly the right policy on this collision-bound trace.
- The one structural hit-rate gain after early saturation came from *geometry,
  not logic*: a single bucket-count halving (capacity conserved) relieved mild
  associativity starvation.
- Behaviour-preserving cost rewrites (O(1) eviction/put paths, smaller `Item`
  metadata, rotate-based promotion/admission) are reliably byte-identical and
  safe to adopt.

## Recurring failures

- Any change that withholds or strips earned protection regresses: counter
  aging, two-hit promotion thresholds, protected-segment caps with demotion,
  and CLOCK recency approximation all erode the durable protection the trace
  rewards.
- Composing the orthogonal levers from early cycles (front admission +
  frequency-first eviction) conflicts rather than stacks — a fresh key becomes
  both recency-oldest and min-frequency, so it is evicted immediately.
- "Recently evicted" is not a reliable proxy for "hot": LFU-min fallbacks and
  ghost-ring fast re-entry let weakly-warranted keys seize protected space.
- Re-tuning protection internals fails: elite/third tiers, reserved
  probationary slots, and looser/larger protected sizing all net-negative.
- A second bucket halving overshoots (routing loss), and an absolute
  associativity target degenerates to a sub-optimal fixed width — geometry is
  bracketed on both sides.
- Stronger hashing (avalanche finalizer + Lemire reduction) gains nothing;
  SipHash is already uniform on this trace.

## Open questions

- Is hit-rate genuinely at its ceiling now that admission, promotion,
  eviction signal, re-entry, routing, tiering, protected sizing, and bucket
  width are all exhausted?
- Should a per-op-time / metadata-size fitness dimension be added? Cost-side
  wins keep recurring and keep landing as undifferentiated ties because the
  hit-rate-only vector cannot see or reward them — this decision has been
  deferred for ~five cycles.
- Did front-admission "evict newest unpromoted" ever actually move hit-rate,
  or was the scan-resistance claim unsupported? Worth a clean re-test before
  recycling.
- Is there any untested trace-structure axis left, or should the trace be
  declared solved and the loop stopped?

## Per-policy notes

- **LRU / recency:** Full positional recency ranking is load-bearing in small
  buckets; coarse one-bit (CLOCK) approximations underperform. Promote-to-MRU
  placement beats seating promotions at the protected-LRU boundary.
- **LFU / frequency:** Useful as the original victim-selection signal, but
  banked counts do not separate hot from rising keys better than protected-LRU
  recency; aging counters and LFU-min fallbacks both regress.
- **Segmented-LRU (current baseline):** FIFO probation + single-hit
  promote-to-MRU + full immortality at current protected size is the settled,
  repeatedly-confirmed policy; internal re-tiering and re-sizing all fail.
- **Geometry / routing:** One bucket halving is the optimum; further width
  change (either direction) and hash-quality changes do not help. Treated as
  an axis independent of, and now exhausted alongside, the segment logic.
