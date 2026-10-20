# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0010

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-05_22-39-10-051/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Forked from cycle 0008's twin winners, all three variants returned to the
hit-rate axis along the admission/eviction mechanics the prior digest pointed
to: an online hill-climbing controller for the protected/probationary split
(v001), making the eviction gate score the incoming key by its pre-increment
frequency (v002), and adding a fixed hysteresis margin to the admission
contest (v003). Both stricter-gate variants — v002 and v003 — took the Winner
slots on barely-perceptible upticks; the adaptive controller regressed.

## Patterns

- The segment boundary is conclusively not the lever, even when made
  dynamic. Static sweeps had already tied across 0.5–0.9; v001 shows an
  adaptive controller does no better and slightly worse — hill-climbing
  `protected_cap` only churns proven-hot nodes back to probation while
  oscillating around an optimum the static sweeps already sat on. Phase
  tracking finds no phases worth tracking on this trace.
- Tightening the admission gate is the one direction that still nudges
  upward, but only at the edge of noise. Both winners *restrict* who can
  evict a resident — v002 by removing the incoming key's self-count so a
  once-seen key can't arrive pre-inflated, v003 by demanding a margin so a
  newcomer must be two observations hotter. Each spares a handful of warm
  probationary residents from marginal, non-reused arrivals.
- The strict-guard theme is now firmly directional. Cycles 0007–0008 showed
  *loosening* gates loses; this cycle shows *tightening* them is the only
  thing that gains. The admission contest sits just slightly looser than
  optimal, but the headroom is tiny — these are correctness/consistency wins,
  not exploitable levers.

## Open questions

- The two winning tightenings (self-consistent scoring, hysteresis margin)
  are partially overlapping fixes to the same gate — do they stack, or do
  they both consume the same handful of tie-or-near-tie contests the way
  victim-sampling and promotion-gating overlapped in cycle 0006?
- The efficiency pivot the last two digests reserved — trimming the
  confirmed-redundant sketch depth or protected slack for free — was again
  skipped in favor of hit-rate. Is there appetite to actually fire it, or
  does the cohort keep reflexively returning to the saturated axis?

## Continuation

Hit-rate is essentially exhausted: the segment boundary is inert even when
adaptive (drop dynamic-sizing ideas entirely), and the only remaining lift is
sub-noise admission tightening. Both winners point the same way — stricter
gates win, looser gates lose — so if hit-rate is pursued at all, test whether
the two gate tightenings (v002 + v003) *combine* rather than proposing a third
independent guard. But the stronger move is the long-deferred efficiency
pivot: cycle 0008 proved sketch depth-8 redundant and protected sizing flat,
so probe trimming depth or protected slack to buy cost for free without losing
the ceiling hit-rate. Stop reaching back to the saturated hit-rate axis.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
