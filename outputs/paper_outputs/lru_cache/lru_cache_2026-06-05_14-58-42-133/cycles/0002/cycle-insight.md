# Cycle 0002 Insight

## Cohort outcome

This cycle tested the prior continuation two ways: layering a ghost
queue onto the proven SLRU base, and exploring a fully adaptive ARC from
baseline. The SLRU+ghost layering regressed hit-rate, while the
self-tuning ARC won — making ARC the sole Winner and the new proven base.

## Patterns

- The promotion gate remains the load-bearing lever. The variant that
  bypassed it — readmitting a returning ghost key straight into the
  protected segment on a single reappearance — over-promoted low-reuse
  keys and displaced hot ones, regressing below its SLRU parent. The
  Winner respects the gate, promoting only on a genuine second reference.
- Self-tuning beat hand-set structure: an adaptive recency/frequency
  target driven by ghost hits outperformed the fixed SLRU split, even
  though it carried more bookkeeping (a second map, ghost arena, four-list
  threading).
- Ghost memory only pays off when it feeds the promotion decision rather
  than overriding it; the same ghost signal helped (ARC) or hurt
  (SLRU+ghost) depending on whether it short-circuited the second-touch
  rule.

## Open questions

- The frequency-sketch admission gate was never actually evaluated this
  cycle (its variant failed to produce a result), so whether a TinyLFU
  estimate adds anything on top of ARC's adaptivity is still untested.
- Overhead stays invisible under hit-rate-only scoring; ARC's extra
  structures would matter if cost is ever measured.

## Continuation

Build the next cycle's hypotheses on the ARC Winner as parent. Drop the
direct-to-protected ghost-readmission framing — it has now been shown to
over-promote. Worth probing: a frequency-sketch admission gate on top of
ARC (the untested idea from this cycle), and varying ARC's ghost-list
bounds or `p` adaptation rate. Avoid reintroducing fixed-split SLRU as a
standalone parent unless a new scored dimension rewards its lower overhead.
