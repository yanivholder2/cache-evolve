## Recurring wins

- Cheap, near-pure hot-path edits pay off: the inline integer hasher won
  on its own (C1) and was still in the winning stack (C2). The cheaper a
  change keeps the universal hot path, the more reliably it lands.
- Hit-rate (segmentation policy) and per-op-time (cheap hasher + skipping
  no-op list surgery) are orthogonal, stackable axes — the best variants
  improve both at once with neither paying for the other.
- The SLRU segmentation lever is a genuine hit-rate win against this
  workload's weakness; once paired with segment-aware fast paths it
  defends the hit-rate record without a latency penalty.
- The narrow-`u32`-link metadata re-encoding does land the smaller node
  when the width casts are centralised rather than sprinkled per-access.

## Recurring failures

- Moving a target dimension by buying it with added per-op latency
  elsewhere loses: naive SLRU promote/demote branching and per-access
  index-width casts both regressed latency in C1.
- Bare data-dependent branches on the universal hot path (MRU guard,
  `tail == idx` skip) mispredict and regress both median and tail — the
  same surgery-skip idea won only when gated on a cheaply predictable
  segment-position condition.
- "Pure representation" sentinel encodings are not latency-free: replacing
  `Option` niches with explicit sentinels still costs measurable median
  and tail even when casts are centralised.

## Open questions

- Can the smaller metadata node and the latency-optimal hot path coexist
  in one variant, or does any sentinel encoding inherently tax per-op-time?
- Are there further no-op-touch surgery skips (same-segment, head) that can
  be gated on cheap position checks without inviting a mispredicting branch?
- The hasher's median win came with a get tail-latency regression — does
  the tail need dedicated attention, or do the fast paths absorb it?

## Per-policy notes

- LRU baseline: weak on hit-rate for this (LIRS-style) workload, which is
  what the segmentation lever targets.
- SLRU: holds the hit-rate record and stacks cleanly with the latency
  levers; only pays off when promote/demote work is gated by cheap
  segment-position checks rather than naive branching.
- MRU guard: lost as a standalone hot-path branch; not worth pursuing in
  bare data-dependent form.
