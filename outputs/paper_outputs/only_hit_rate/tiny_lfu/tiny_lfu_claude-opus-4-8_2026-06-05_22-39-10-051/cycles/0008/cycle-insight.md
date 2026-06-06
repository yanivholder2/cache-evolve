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
