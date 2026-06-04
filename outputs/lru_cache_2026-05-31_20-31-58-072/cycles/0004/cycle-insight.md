# Cycle 0004 Insight

## Cohort outcome

The cohort obeyed cycle 3's directive and attacked the pass-1 / Cold
side from three angles — scan-taint pre-pass on brand-new admissions
(v001), least-recently-touched victim selection within the Cold
ref-clear pool (v002), and a two-touch Warm promotion ladder (v003) —
and all three landed at exact parity with the cycle-3 champion on
lirs:multi1@512. All three sit on the Pareto front by tie, but the
working champion is unchanged.

## Patterns

- Three independent edits to the eviction loop all produced
  bit-identical hit-rate. The plateau hypothesised at the end of
  cycle 3 is now strongly confirmed: at this capacity on this trace,
  the residual loss channel is not reachable by re-ranking Cold
  victims or hardening Hot admission.
- v001 and v002 both *re-ordered* the existing victim pool; the
  cohort shows that swapping which doomed Cold slot dies first does
  not change net hit-rate, because the spared slot is itself reached
  within a sweep or two and the re-reference distribution doesn't
  land in that window.
- v003 changed *which slots become Hot* by requiring two touches, but
  the new Warm-victim sub-state simply absorbs whatever the Hot pool
  no longer holds — net victim selection on this trace is unchanged.
- The Cold→Hot machinery's *existence* (ghost + promotion class) is
  doing nearly all of the lift on this workload; the shape of the
  inside of that machinery, across two cycles of refinement, is now
  inert.
- The fitness-vector bottleneck (no per-op-time, no metadata-size)
  flagged in cycles 1–3 is still unaddressed; this cycle's parity
  outcomes make that meta-bottleneck operationally acute, because
  there is now nowhere left to move on the one scored axis.

## Open questions

- Has lirs:multi1@512 simply been wrung out for this skeleton? If so,
  any further single-workload variant on it will keep scoring at
  parity and the loop will stall regardless of policy creativity.
- Would the inert knobs (taint pre-pass, intra-Cold LRU, Warm ladder)
  actually differentiate under different capacities or trace mixes,
  where the Cold pool churns faster or pass 2 fires more often?
- Is the next productive move policy at all, or is it scoring /
  workload diversification — i.e. is the cohort's repeated parity a
  policy-design problem or an evaluation-setup problem?

## Continuation

The cohort delivered the clearest plateau signal yet: three orthogonal
pass-1 / Cold-side edits, all hit-rate-neutral. Next cycle should
stop spending slots on further single-trace, single-capacity policy
refinements of the CLOCK-Pro-lite skeleton — pass-1 victim-pool
re-ranking, scan filters on brand-new admissions, and promotion-ladder
hardening have now each scored at parity. The cycle-3 v001 (full-cap
ghost ring) remains the working champion by default. Hypothesis-writers
should pivot effort toward (a) widening the evaluation surface so
policy differences can show up at all — additional capacities of
lirs:multi1, or different traces, sampled at matched sizes per the
cycle-1/2 sampling-discipline lesson — and/or (b) finally addressing
the fitness-vector narrowness so the inert-on-hit-rate edits (v002,
v003-style structural changes) can be scored on the axes they actually
move. Further single-axis, single-workload tuning of this skeleton
should be deferred until one of those two shifts happens.
