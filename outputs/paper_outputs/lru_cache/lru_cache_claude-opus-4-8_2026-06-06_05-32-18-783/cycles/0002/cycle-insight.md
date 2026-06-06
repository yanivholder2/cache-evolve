# Cycle 0002 Insight

## Cohort outcome

This cycle revisited the two open levers from Cycle 0001 and probed a
hot-path skip. The standout Winner combined the SLRU hit-rate policy
with the integer hasher and segment-aware fast paths, defending the
record hit-rate while pulling latency back to hot-path-class — proving
the policy and latency levers are orthogonal and stackable. The
narrow-`u32`-link metadata re-encoding also landed as a Winner, hitting
the smaller node, while a standalone MRU-guard branch lost.

## Patterns

- Hit-rate (segmentation policy) and per-op-time (cheap hasher + skipping
  no-op list surgery) are independent axes: the cycle's best Variant
  optimized both at once with neither paying for the other.
- Skipping redundant list surgery only pays when the skip condition is
  cheaply predictable. Conditioned on segment position (already-MRU within
  protected) it helped; as a bare data-dependent `tail == idx` branch on
  the universal hot path it mispredicted and regressed both median and
  tail — the same idea won in one framing and lost in another.
- The narrow-link metadata win is genuinely achievable and centralising
  the casts keeps the slowdown modest, but re-encoding `Option` niches as
  explicit sentinels still costs measurable median and tail latency — a
  "pure representation" change is not actually latency-free.

## Open questions

- Can the metadata (24-byte node) and the latency-optimal hot path coexist
  in one Variant, or does any sentinel encoding inherently tax per-op-time?
- The winning fast path skipped surgery using a predictable position
  check — are there other no-op-touch cases (same-segment, head) skippable
  the same cheap way without inviting a mispredicting branch?

## Continuation

Carry forward the combined SLRU-plus-fast-hasher Variant as the new base —
it holds both the hit-rate record and competitive latency. Next cycle
should try folding the narrow-`u32`-link metadata win onto that base and
measure whether the encoding's latency tax survives once the fast paths
absorb it. Drop bare data-dependent MRU-guard branches on the universal
hot path; only pursue surgery-skips gated on cheaply predictable position
conditions.
