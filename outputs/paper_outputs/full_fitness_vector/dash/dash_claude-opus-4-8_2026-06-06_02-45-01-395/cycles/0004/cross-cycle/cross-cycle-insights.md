# Cross-Cycle Insights

## Recurring wins

- Latency wins are repeatable when they remove real hot-path cost at a
  resolvable granularity: structural changes like devirtualizing the policy
  call and enabling inlining (c4) land above the noise floor, as did removing
  O(n) front-shifts and per-hit syscalls (c1).
- Per-entry footprint shrank cleanly by deleting metadata the active
  positional policy never reads (c1→c2), and placement vs. footprint proved
  genuinely orthogonal — slimming `Item` stacked onto MRU promotion with no
  hit-rate cost (c2).
- The container/segment pivot succeeded where per-entry tuning stalled:
  relocating the heat signal into a segment-side map reached the long-sought
  Pareto corner — best hit-rate at the metadata floor (c4).

## Recurring failures

- Tuning the promotion gate (width, fraction, positional vs. frequency)
  saturated quickly: MRU placement already captures the warm mass, so gate
  shape and placement are substitutes, not additive (c2→c3).
- Chasing hit-rate through the hot `get` path costs latency for noise-level
  gain — wider-gate cascades and per-hit counter writes both raised p50/p99
  for nothing (c3).
- Micro-latency reorders (single scan/branch per hit) are below harness
  resolution and moved the wrong way under variance (c3).
- Raising set-associativity left hit-rate bit-for-bit unchanged, falsifying
  the conflict-miss hypothesis (c4).

## Open questions

- Is any lever left that can move hit-rate, or is it fully capped by the
  trace's compulsory/capacity mass now that placement, gate shape, and
  associativity are all exhausted?
- Do the devirtualization latency win and the side-table size win stack, or
  does the side-table's added hot-path hashing eat the latency gain?
- Are there further structural latency wins of resolvable magnitude
  (allocation strategy, bucket layout), and can any reduce p99 rather than
  just p50?

## Per-policy notes

- `ClassicLRU` (active eval policy throughout): purely positional, so it
  ignores recency/frequency metadata — every `Item` field beyond key+value
  was dead weight removable for free. Its hit-rate is capacity/compulsory-
  bound, not conflict-bound, so placement and gate tuning saturate fast and
  the durable wins come from layout/structure, not policy cleverness.
- Side tables (segment-owned heat map) trade tail latency for size: the free
  struct read becomes a hash-and-probe, pushing p99 up — relocation is not a
  free substitute.
