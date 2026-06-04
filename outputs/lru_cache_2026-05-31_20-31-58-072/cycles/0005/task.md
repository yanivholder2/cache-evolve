# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0005

## Pareto front

- v001
- v002

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0005/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0005/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0005/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0005/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-05-31_20-31-58-072/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

All three variants targeted the only scored axis (hit-rate), responding
to cycle 1's call to compete on it. Two built on v002's CLOCK base
(GCLOCK aging counter, CLOCK-Pro-lite cold/hot + ghost ring) and one
rebuilt SLRU on the baseline arena. Only CLOCK-Pro-lite (v002) landed on
the Pareto front; GCLOCK regressed and the SLRU rebuild again failed to
establish a matched-size delta.

## Patterns

- Among CLOCK extensions, what the extra per-slot state encodes
  determined the sign of the result: state that records recency (Cold→Hot
  on second touch, plus evicted-key memory) helped, while state that
  records frequency (saturating aging counter) hurt. On a recency-skewed
  trace, frequency-leaning bits actively shield stale-popular keys and
  cold-scan inserts.
- Layering complementary mechanisms beat widening a single signal:
  combining a promotion class with a small ghost ring closed two
  distinct gaps (re-reference protection and just-evicted recall) that
  no single-axis tweak addressed.
- The cohort hit cycle 1's same-size comparability trap a second time:
  the SLRU variant ran at a capacity disjoint from its baseline's
  sampled sizes, so the SLRU-vs-LRU question is still open after two
  cycles of attempts.
- The fitness-vector bottleneck flagged in cycle 1 (no per-op-time or
  metadata-size dimensions) is still in force; the cohort sidestepped it
  by uniformly competing on hit-rate rather than addressing it.

## Open questions

- Is CLOCK-Pro-lite's win specific to lirs:multi1, or does the
  promotion-plus-ghost-ring pairing hold on traces with different
  re-reference-distance profiles?
- Can the SLRU mechanism ever be evaluated honestly without first fixing
  the sampling discipline so parent and child share capacities?
- What further refinements to CLOCK-Pro-lite (ghost-ring sizing, Hot
  demotion aggressiveness, second-pass policy) extend the win without
  re-introducing the frequency-bit trap GCLOCK fell into?

## Continuation

Treat cycle-2 v002 (CLOCK-Pro-lite) as the new working champion and
build the next round on it: tune ghost-ring size, explore alternative
Hot-demotion rules, or layer a recency-anchored admission filter. Drop
the GCLOCK / frequency-counter lineage entirely until other workloads
are scored — on a recency-skewed trace it is a structural loss. Do not
queue another SLRU attempt unless the next cycle can guarantee matched
capacities between variant and baseline samples; otherwise the result
will be uninterpretable for the third time. The fitness-vector
narrowness remains the unaddressed meta-bottleneck and should be named
again as a constraint for hypothesis-writers.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

All three variants tuned the cycle-2 CLOCK-Pro-lite champion along
orthogonal axes — ghost-ring size (v001), Hot-demotion stickiness
(v002), and ghost provenance plus warm re-admission (v003) — and only
v001 landed on the Pareto front, and only by a marginal lift. The two
policy-shape variants were inert: their machinery sits on a code path
that this trace/size barely exercises.

## Patterns

- The champion's loss channel on lirs:multi1@512 is concentrated on
  pass 1 (Cold sweep) of the eviction loop, not pass 2. v002 and v003
  both edited pass-2 / Hot-side mechanics (sticky demotion, Hot-ghosts,
  warm re-admit) and produced exactly the parent's score — pass 2
  simply does not fire often enough at this cap for those knobs to
  bite.
- Among the three knobs, the only one that moved the needle was
  monotonic-magnitude tuning on a mechanism already in the hot path
  (ghost length). Even there, quadrupling the ring caught only a
  sliver, indicating the re-reference distribution past the
  quarter-capacity mark has a thin tail.
- The cohort confirms cycle-2's reading from the other side: the
  champion's win came from the *existence* of a ghost and a
  promotion class, and further marginal returns on the same workload
  are small. We are climbing toward a local plateau on this trace.
- The fitness-vector bottleneck flagged twice now remains unaddressed;
  the cohort again competed only on hit-rate.

## Open questions

- If pass 2 is dormant at cap=512, would Hot-side knobs (H02-style
  sticky demotion, H03-style Hot-ghosts) actually pay off at smaller
  capacities or on traces where the Cold pool runs out and pass 2 is
  forced to fire?
- What are the pass-1 levers we have not yet pulled — Cold-admission
  filtering, ghost-hit promotion targeting, or alternative
  scan-resistance heuristics on the Cold sweep itself?
- Is the lirs:multi1 trace itself now the limiting factor — i.e. how
  much more hit-rate is recoverable on this single workload before
  diversity of traces becomes the bottleneck instead of policy?

## Continuation

Treat v001 (full-capacity ghost ring) as the new working champion but
flag that the ghost-sizing knob is in strong diminishing returns;
further size sweeps are not worth a slot. Drop the Hot-side
modification lineage (H02 sticky demotion, H03 Hot-ghosts, hot_credit)
on this trace/size — three variants now show pass 2 is dormant here,
so any pass-2-only edit will keep scoring at parity. Direct next-cycle
hypothesis-writers to pass-1 / Cold-side mechanics instead: how Cold
victims are chosen, how ghost-hits seed Hot state on re-admission, or
Cold-admission gating that filters scan inserts before they pollute
the ring. The fitness-vector narrowness (no per-op-time, no
metadata-size) remains the unaddressed meta-bottleneck and should be
named again as a constraint; the cohort is otherwise about to plateau
on hit-rate-of-one-trace.


### Cycle 0004

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
