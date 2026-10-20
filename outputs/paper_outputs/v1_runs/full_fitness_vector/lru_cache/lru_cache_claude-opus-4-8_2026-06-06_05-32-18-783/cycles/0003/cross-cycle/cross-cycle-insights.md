## Recurring wins

- Cheap, near-pure hot-path changes pay off: the inline integer hasher
  improved per-op-time directly with no hard regression and even shed a
  default-hasher seed for a free metadata side-win.
- Hit-rate (segmentation policy) and per-op-time (cheap hasher + skipping
  no-op list surgery) behave as orthogonal, stackable axes — the strongest
  variants optimized both at once with neither paying for the other.
- The combined SLRU-plus-fast-hasher base has held the hit-rate record and
  competitive latency across two cycles and remains the undisplaced champion.

## Recurring failures

- Single-axis levers that buy their target dimension with added per-op
  latency lose: extra promote/demote branching and per-access width casts
  both moved their axis but regressed latency.
- Bare data-dependent guard branches on the universal hot path mispredict
  and regress both median and tail; the same surgery-skip idea wins only
  when gated on a cheaply predictable position.
- "Pure representation" sentinel re-encodings are not latency-free —
  narrow-`u32` links land the smaller node but still cost measurable
  median/tail even with centralized casts.
- Unconditional ghost re-admission straight into protected over-promotes,
  evicting genuinely hot keys and dropping hit-rate below plain SLRU while
  paying per-put bookkeeping latency.

## Open questions

- Can the 24-byte-class metadata node and the latency-optimal hot path
  coexist? The narrow-link fold onto the SLRU base remains genuinely
  untested (its variant died on infra, not merit) and is the top open lever.
- Does gentler ghost admission — probation re-entry or repeated-hit gating
  rather than single-hit promotion — recover the reuse-distance signal
  without breaking hit-rate?
- Are there other no-op-touch cases (same-segment, head) skippable as
  cheaply as the protected-MRU case without inviting a mispredicting branch?

## Per-policy notes

- Segmented LRU (SLRU): the durable hit-rate lever for the LIRS workload;
  its protected segment is the one place the surgery-skip reliably pays
  because the touched node is reliably the MRU there.
- Plain single-list LRU: the surgery-skip generalizes poorly here — the
  touched node is rarely the tail, so the guard mispredicts; keep the skip
  segment-gated only.
- Reuse-distance / ghost-history re-admission: right intuition for the
  trace, but unconditional promotion overshoots; needs gentle admission to
  be viable.
