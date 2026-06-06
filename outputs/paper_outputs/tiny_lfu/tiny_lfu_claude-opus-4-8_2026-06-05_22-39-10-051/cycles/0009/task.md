# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0009

## Winners

- v002
- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Forked from cycle 0005's promotion-gate winner, all three variants stayed
on the SLRU axis and probed where else the sketch signal could steer
movement: stacking victim sampling onto the promotion gate (v001), applying
sampling to the protected-tier *exit* / demotion choice (v002), and adding a
floor threshold on protected entry (v003). v002 took the Winner slot on a
hair-thin lift; v001 added nothing over its parent, and v003 was an
inadvertent no-op.

## Patterns

- The compounding bet from cycle 0005 is now answered and refuted: victim
  sampling and frequency-gated promotion do *not* stack. Both mechanisms fire
  on the same scarce "head hides a hotter key" events, so layering them
  overlaps rather than adds — confirming they target one shared rare case,
  not disjoint ones.
- The one new placement for the sketch signal — the protected-tier *exit*
  (which node to demote on overflow) — is directionally right and edged
  out the parent, but only because it occasionally spares a frequency-hot,
  recency-stale resident. Its room to move is tiny: protected recency order
  already tracks sketch frequency closely, the same workload signature seen
  across cycles 0003–0005.
- A "lever" that never actually engages produces exactly parent results. The
  entry-floor variant was inert not because the idea is wrong but because
  increment ordering made the gate unsatisfiable — a reminder that null
  results need a mechanism check before they count as evidence about the
  workload.

## Open questions

- Every distinct point where the sketch can steer SLRU movement — admission,
  promotion entry, promotion gating, probationary eviction, protected
  demotion — has now been touched and each is marginal-or-inert. Is there any
  structural placement left untested, or is the hit-rate axis fully mapped?
- The exit lever (v002) won this cycle; would a stronger entry gate (a floor
  that actually fires, e.g. evaluated pre-increment) reserve protected slots
  in a way the current always-true entry does not?

## Continuation

The SLRU hit-rate axis is now comprehensively mapped: admission, promotion,
victim choice, and demotion have all been probed and every lever is real but
marginal or redundant. The compounding hypothesis is refuted — sampling and
promotion-gating overlap, do not stack. Across cycles 0003–0006 the evidence
that this workload's hit-rate sits at its ceiling is now very strong. Next
cycle should stop chasing hit-rate through structural refinement and pivot
decisively to efficiency/cost dimensions; if any hit-rate probe is attempted,
make it a corrected entry-floor (one that can actually fire), not another
victim/demotion sampler.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle finally fired the two probes the prior digests had reserved: a
*corrected* pre-increment entry floor (v001), a direct-to-protected admission
placement (v002), and the long-requested efficiency pivot — a 4x-narrower
sketch leaning on conservative update (v003). v001 took the Winner slot but
only by holding exactly at parent; v002 regressed a hair and v003 regressed
measurably. No lever produced lift.

## Patterns

- The corrected entry floor now engages mechanically, settling cycle 0006's
  inert-gate ambiguity — but it gates nothing on this trace: the probationary
  keys that earn a warm-up hit have already been seen twice, so the floor
  admits exactly the keys the unconditional parent admitted. A live lever with
  no work to do is the cleanest confirmation yet that the hit-rate ceiling is
  structural, not a missed gate.
- Steering placement from the *admission* side (v002) is net negative: a single
  sketch estimate is a weaker signal than an observed in-cache hit, so bypassing
  probation admits occasional false-hot keys. The probationary "earn a hit"
  filter is doing real work and removing it costs more than it saves.
- The efficiency pivot has a hard floor: conservative update does *not* hold
  accuracy at 1/4 width. The extra sketch width was not waste — this workload
  genuinely needs more cells than capacity×1 to separate its distinct keys, so
  the cheapest-cache-for-free route via narrowing is closed.

## Open questions

- With admission, promotion entry/gating, eviction, demotion, *and* a working
  entry floor all now probed and inert, is there any hit-rate lever left
  untouched, or is the structural axis fully and finally mapped?
- v003 refutes 4x narrowing but not the existence of *some* width below the
  current setting that still holds accuracy — is there a shallower memory
  reduction (e.g. 2x) that trades cells for cost without losing hits?

## Continuation

The hit-rate axis is now exhaustively mapped and confirmed at its ceiling: even
a correctly-firing entry floor finds nothing to gate, and admission-side
placement actively hurts. Stop proposing structural hit-rate refinements — every
SLRU steering point is spent. The efficiency pivot is now live but bounded: 4x
narrowing is refuted, so do not retry aggressive width cuts; if cost is pursued,
probe a *shallow* reduction (2x) to find where accuracy actually breaks, or turn
to other cost/efficiency dimensions entirely rather than memory width.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

All three variants forked from cycle 0007's hold-at-parent winner and
probed the last untouched corners of two already-mapped axes: a fresh
estimator dimension — sketch *depth* (rows), never before swept (v001);
SLRU protected sizing pushed *above* 0.8 for the first time (v002); and a
coherence fix aligning the promotion gate with its actual demotion victim
(v003). v001 and v002 took the Winner slots, each on a within-noise uptick;
v003 regressed and is refuted.

## Patterns

- The estimator is now saturated on *both* of its axes. Cycles 0002–0007
  exhausted sketch width; v001 shows depth is equally inert — doubling the
  rows whose min is taken removes essentially no false-hot admissions. The
  two orthogonal collision-reduction mechanisms are independently confirmed
  spent, so estimator fidelity is fully and finally mapped.
- SLRU protected sizing is now a confirmed flat plateau at the top. The
  long-reserved "go larger than 0.8" probe finally fired and tied parent at
  0.9: the keys saved by the extra protected slots are nearly the same ones
  probation would have retained anyway. The favored direction from cycle
  0005 holds, but its curve is flat — there is no exploitable lift above 0.8.
- Loosening a conservative gate to admit "more proven-warm" keys keeps
  losing. v003 swapped the strict head-based promotion contest for the
  sampled-coldest-victim's frequency; like the admission-side placement in
  cycle 0007, widening admission into marginal warming keys trades a
  proven-warm resident for a less-certain promotee and costs hits. The
  stricter guards are doing real work, not hiding latent bugs.

## Open questions

- With estimator width *and* depth, every SLRU steering point, and now
  protected sizing above 0.8 all probed and inert, is any hit-rate lever
  left untouched at all — or is the entire search space conclusively mapped?
- v001/v002 won only by holding at-or-near parent; is there still an
  efficiency/cost dimension where saturation is an *opportunity* (e.g.
  trimming the now-confirmed-redundant sketch depth or protected slack for
  free) rather than a dead end?

## Continuation

The hit-rate axis is now exhaustively and conclusively mapped: estimator
fidelity is saturated on both width and depth, every SLRU steering point is
spent, and protected sizing is flat above 0.8 — stop proposing hit-rate
refinements entirely, all three of this cycle's positives were ties. Do not
revisit gate-loosening (v003-style) — strict guards keep winning. The clear
remaining move is the efficiency pivot, now with new leverage: cycle 0007
showed accuracy needs the current width, but v001 just proved depth-8 is
redundant — so probe whether *reducing* depth (back toward 4 or lower) or
trimming protected slack buys cost for free without losing the now-confirmed
ceiling hit-rate.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
