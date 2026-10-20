# Cycle 0003 Insight

## Cohort outcome

This cycle ran the partition-ratio sweep to its end (H01, ~4/96) and probed
two orthogonal levers off the ~6/94 parent: a freshness-gated deeper ghost
queue (H02) and a stricter small-to-main promotion gate (H03). Only the
final partition shrink (v001) landed in the Winners, and only by a sliver;
the freshness gate regressed and the stricter promotion gate was completely
inert.

## Patterns

- The partition-ratio sweep has effectively flattened. v001 still edged the
  parent at ~4/96, but the gain was a fraction of the prior step's — the knee
  is here. The workload tolerates a very thin admission filter without
  leaking one-hit-wonders, but there is no more meaningful retention to
  extract by enlarging main.
- Shrinking main's admission filter this far collapsed the promotion-gate
  lever entirely: with so small a small-queue, entries are evicted FIFO
  before they can sit at the tail as borderline two-hit keys, so `freq > 1`
  vs `freq > 2` is a no-op. The partition knob has absorbed the promotion
  knob — distinct-on-paper levers keep collapsing into sizing.
- The "every ghost hit is strong reuse evidence, send it direct-to-main"
  finding now holds across all three tried gating axes (small-queue
  re-admission, throttling, and position/freshness). The untried middle path
  from cycles 0001–0002 also failed: splitting ghost hits by recency
  discards a confirmed signal as fast as the deeper window earns it.

## Open questions

- With partition sizing, promotion threshold, and ghost re-admission gating
  all exhausted, what lever is genuinely orthogonal to main-queue retention?
  Candidates not yet meaningfully moved: ghost *sizing* on its own (without a
  re-admission gate), or the second-chance/frequency mechanics themselves.
- Is the remaining headroom on this workload small enough that the current
  ~4/96 configuration should simply be frozen as the operating point?

## Continuation

Freeze the partition ratio at ~4/96 — the sweep is done and the promotion
gate is now inert beneath it, so drop both. Abandon all ghost re-admission
gating (freshness, throttling, small-queue routing all regressed); the
direct-to-main path is correct and every variant on this axis has failed.
The next cycle should stop tuning main-queue retention entirely and seek a
lever on a different mechanism, or accept the configuration as converged.
