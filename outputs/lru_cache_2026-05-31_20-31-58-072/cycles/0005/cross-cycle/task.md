# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

The cohort attacked three independent axes off the baseline LRU: segment
structure (v001 SLRU), replacement policy (v002 CLOCK), and link
representation (v003 u32+sentinel). Only v002 landed on the Pareto
front, because hit-rate is currently the sole scored dimension and CLOCK
was the only variant whose hit-rate signal was both non-trivial and
favourably positioned against the baseline sample.

## Patterns

- Two of three variants (v002, v003) made bets whose intended payoff
  lives on dimensions the fitness vector does not yet score (per-op time
  for CLOCK, metadata size for the u32 sentinel). Their structural wins
  are real but invisible to selection.
- The one variant whose bet did target hit-rate (v001 SLRU) showed a
  scan-resistance win in qualitative terms, but the variant and baseline
  were sampled at different cache sizes, so a same-size delta over LRU
  was not actually demonstrated.
- All three variants preserved the arena-slot-reuse pattern and the
  P1/P2/P3 invariants cleanly, suggesting the baseline's structural
  scaffolding is permissive of large representational rewrites.

## Open questions

- Without per-op time and metadata-size dimensions scored, how can the
  loop distinguish a real structural win (v002, v003) from neutral
  noise?
- Is v001's apparent hit-rate movement actually attributable to SLRU, or
  is it a sampling artefact of the size/trace pairing?
- Does CLOCK's slight recency-fidelity loss matter on traces less
  recency-skewed than LIRS:multi1?

## Continuation

Next cycle should treat the fitness vector itself as a bottleneck: until
per-op time and metadata size are scored, variants like v002 and v003
cannot be properly selected and the cohort risks collapsing onto
hit-rate-only tweaks. Hypothesis-writers should either (a) build on
v002's CLOCK base with hit-rate-recovering refinements (e.g.
two-handed CLOCK, CLOCK-Pro-lite) that compete on the one scored axis,
or (b) explore SLRU-style segmentation parameter sweeps anchored to a
shared baseline size so the same-size delta is actually measurable.
Drop pure representation-only repacks (v003-style) until a metadata-size
dimension exists.


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


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort tried one genuinely new direction the cycle-4 cycle-insight
called for — a full skeleton swap to S4LRU (v003) — alongside two more
single-knob edits to the CLOCK-Pro-lite champion (cold-admission
bypass-on-first-sight in v001, ghost-ring-position-weighted promotion
in v002). The two single-knob edits tied the champion at parity (so
they sit on the Pareto front by tie); the skeleton swap regressed
below the plateau. The working champion is unchanged.

## Patterns

- The plateau flagged in cycle 4 held a second time: no variant moved
  hit-rate up. Three consecutive cohorts on this trace/size have now
  failed to lift the scored axis.
- The one variant that swapped skeletons (S4LRU) lost ground. A
  multi-hop promotion ladder paired with conservation-by-demotion
  shrinks effective top-level capacity on a trace whose re-reference
  distance does not survive three intermediate hops — recency
  segmenting *without* a ghost ring or an asymmetric (top-heavy) split
  is strictly worse than the current ghost-anchored skeleton on
  lirs:multi1@512. This is the first cycle that gives the champion
  structural credit, not just plateau credit: the ghost ring and the
  single-hop Cold→Hot promotion are doing real work.
- Ghost-ring *position* (v002) is not a usable freshness gradient
  here: freshest-quarter ghost-hits behaved identically to older ones.
  Differentiated promotion strength keyed on ghost position is a dead
  knob on this workload.
- v001's parity is a false negative, not a verdict on the lever: the
  first-seen sketch was marked on `get` *before* `put` read it, so
  under the framework's miss-then-put pattern the gate's precondition
  was already false by the time it could fire. Cold-admission gating
  itself — the lever the cycle-3 cycle-insight named — was therefore
  not actually exercised this cycle.
- The fitness-vector narrowness (no per-op-time, no metadata-size)
  flagged in cycles 1–4 remains unaddressed, and is now joined by the
  related unaddressed lever from cycle 4: widening the evaluation
  surface to other capacities or other traces. Both calls went
  unanswered this cycle.

## Open questions

- Would a *correctly instrumented* cold-admission gate (sketch marked
  only on `put`, or the gating decision threaded into `get`) actually
  lift hit-rate, or would it too come back at parity once the
  precondition-collapse bug is fixed?
- Is any non-CLOCK-Pro-lite skeleton viable on this trace, or does
  every alternative need at least the ghost ring and the single-hop
  promotion to keep up? Would S4LRU with an asymmetric top-heavy split
  or a ghost ring grafted on close the gap?
- After three plateau cohorts and one structural regression, is the
  loop's productive bottleneck still policy design at all, or has it
  decisively shifted to evaluation-surface / fitness-axis breadth?

## Continuation

Keep cycle-3 v001 (full-cap ghost ring) as the working champion. Drop
two lineages outright: ghost-ring positional weighting (v002 confirmed
inert), and unaugmented multi-level segmented LRU (v003 confirmed
below the plateau — do not retry S4LRU without either an asymmetric
top-heavy split or a ghost-ring augmentation). The one single-trace
policy lever still genuinely unpulled is cold-admission gating: the
H01 mechanism was not actually exercised because of a sketch-marking
bug, so a corrected re-attempt (mark sketch only on `put`, or thread
the gating decision through `get`) is worth exactly one slot — not
more. Beyond that one retry, hypothesis-writers should stop spending
slots on further single-trace, single-capacity refinements and finally
act on cycle 4's directive: widen the evaluation surface (more
capacities of lirs:multi1 or additional traces, sampled at matched
sizes) and/or address fitness-vector narrowness so structural changes
can be scored on axes other than hit-rate. With one cohort having now
confirmed the champion's structural choices are load-bearing (S4LRU
lost), evaluation-surface breadth is the highest-leverage remaining
move.


## Write

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
