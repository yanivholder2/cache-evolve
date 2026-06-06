## Recurring wins

- Cheap, near-pure hot-path changes that *remove* work pay off repeatedly:
  the inline integer hasher, segment-gated surgery skips, and the metadata
  re-encoding all repaid themselves without a paying axis once they reduced
  work instead of adding a branch.
- Hit-rate (segmentation/admission policy) and per-op-time (cheap hasher +
  skipping no-op list surgery) behave as orthogonal, stackable axes — the
  strongest variants optimized both at once with neither taxing the other.
- Levers that lose on first contact tend to land once reframed: the
  narrow-`u32` metadata fold and the reuse-distance hit-rate signal both
  failed early but later won when the cast was centralized and admission was
  gated on repeated evidence.

## Recurring failures

- Buying a target dimension with added per-op latency keeps losing: wider
  index casts, blunt protected-cap widening (80→90%), and extra
  promote/demote branching all moved their axis but regressed latency.
- Bare data-dependent guards on the universal hot path (e.g. `tail == idx`)
  mispredict and regress both median and tail — the surgery-skip only works
  when gated on a cheaply predictable, actually-common position.
- Unconditional ghost-to-protected re-admission overshoots: it evicts
  genuinely hot keys, drops hit-rate below plain SLRU, and still pays
  per-put bookkeeping.

## Open questions

- Can the 24-byte node layout and the gated-ghost admission win be folded
  onto one base, the way SLRU and the fast hasher stacked? They touch
  disjoint structures (node layout vs. external ghost store).
- Is there more retention headroom from admission cleverness (tuning the
  repeated-hit threshold) now that raw cap-widening is shown to cost latency?
- Are there other no-op-touch cases (same-segment, head) skippable as cheaply
  as the protected-MRU case without inviting a mispredicting branch?

## Per-policy notes

- Segmented LRU (SLRU): the durable champion base; carries the hit-rate
  record and supports the protected-MRU fast path that single-list LRU
  cannot.
- Plain single-list LRU: the surgery-skip is fundamentally segment-local and
  does not generalize here — the touched node is rarely the tail, so the
  guard mispredicts.
- Ghost/reuse-distance admission: right intuition for the LIRS trace but only
  safe when gentle — route single hits to probation and gate on repeated
  evidence rather than promoting straight to protected.
- Segment geometry vs. admission: moving the protected/probation split trades
  hot-path latency, whereas smarter admission gains hit-rate for free.
