# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0010

## Winners

- v003
- v001
- v002

## Variant insights

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0010/variants/v001/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0010/variants/v002/insight.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0010/hypotheses/H01.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0010/hypotheses/H02.md
- /home/user/cache-evolve/outputs/tiny_lfu_2026-06-06_02-49-34-396/cycles/0010/hypotheses/H03.md

## Prior cycles digest

### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

A cost-and-refresh cycle: all three variants branched off cycle-0005/v001
(compact layout + wide sketch) and probed whether the working base could
shed work or refresh its signal without losing hit-rate — trimming sketch
depth (v001), shrinking the victim sample (v002), and adding amortized
counter aging (v003). The two trim probes held hit-rate flat and each
banked one axis-local saving; the aging probe confirmed a long-standing
suspicion negatively.

## Patterns

- On the factor-16 sketch, depth and sample-depth are both past
  saturation for *accuracy*: dropping a min-row or cutting the victim
  sample left hit-rate essentially flat, confirming the wide sketch's
  estimate — not row count or walk length — is the binding constraint on
  admission and eviction quality. Their value is now purely cost-side:
  one buys footprint (a quarter of the sketch heap), the other shaves the
  put p99 tail.
- The predicted per-op-time *speedups* did not materialize. Removing one
  counter touch per access and two `estimate` calls per eviction both sat
  below the benchmark noise floor, so the summary per-op metric moved on
  run-to-run variance, sometimes the wrong way. At this width these knobs
  are size/tail levers, not hot-path-speed levers.
- Counter aging is settled as a net loss on this trace. Even tuned wide
  (interval = sketch width, single amortized sweep), halving decayed
  earned counts for a stable hot set faster than it refreshed contrast,
  dropping hit-rate while taxing every `increment`. The base's
  monotonically inflating counters were never the bottleneck here.

## Open questions

- With depth and sample-depth confirmed accuracy-saturated, is footprint
  (drop a row) worth banking permanently, or kept in reserve since it
  doesn't touch hit-rate?
- Is there any remaining lever that sharpens the wide sketch's estimate
  itself — the one thing shown to actually move hit-rate — that hasn't
  already been ruled flat or priced?

## Continuation

Close the aging thread for good: classic counter halving is a confirmed
loss on this stable-hot-set workload — do not revisit it. Treat sketch
depth and victim-sample depth as accuracy-saturated; tune them only for
footprint or tail, never expecting hit-rate or per-op speed. The frontier
is unchanged and narrowing: the only demonstrated hit-rate mover is
estimate sharpness (conservative update from Cycle 0006 remains the live
candidate), and the cheap free-lunch levers are exhausted. Next cycle
should test estimate-sharpening edits on the wide-sketch base rather than
turning any saturated size knob further.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

A decisive lever-exhaustion cycle: all three variants branched off
cycle-0005/v001 (compact layout + wide sketch) and each attacked one of
the three surfaces the cross-cycle log had named as the only live
hit-rate movers — the estimate rule (conservative update, v001), the
admission decision (a one-count margin, v002), and sketch geometry
(width 16→32, v003). None produced a real hit-rate gain; v002's margin
nudged it up by a hair, the other two landed flat. They are Winners only
in the cost-axis sense, not on hit-rate.

## Patterns

- The three named hit-rate levers are now all spent on the wide base.
  Conservative update's factor-4 gain does not transfer to factor-16
  (the wide table already suppressed the aliasing it corrects), width is
  saturated (doubling buys only cache-line tax), and a single-count
  admission margin rarely flips a decision on a hot set whose
  frequencies are separated by far more than one count. Each probe
  closed its surface negatively.
- The binding constraint is now isolated by elimination, not by a win:
  it is neither the admission margin nor the increment rule nor more
  columns. The hot set's true frequencies are already cleanly separated
  at this geometry, so sharpening, hysteresis, and width all push on a
  constraint that is no longer binding.
- Cost asymmetry is now the rule for any estimate/geometry edit: every
  probe that touched the hot path (conservative min-scan, wider stride)
  paid unconditional per-op time for zero hit-rate, while the admission
  margin — which only runs on the full-cache eviction branch — was
  genuinely free but also nearly inert.

## Open questions

- With all three logged hit-rate surfaces ruled flat on the wide base,
  is there *any* remaining edit that moves hit-rate, or has this trace's
  hit-rate effectively saturated at the incumbent base?
- If hit-rate is saturated, does the frontier shift entirely to banking
  cost-axis savings (footprint, tail, per-op) on a fixed hit-rate?

## Continuation

Stop turning the three named hit-rate levers: conservative update is now
ruled flat on the wide base (do not re-test it here), sketch width is
saturated at factor 16, and a small admission margin is inert on this
stable hot set. Cycle 0007 already retired aging, depth, and
sample-depth as accuracy levers. The cross-cycle log no longer points to
a credible hit-rate mover — next cycle should either hunt a genuinely
new admission/eviction *structure* (not a margin or constant tweak) or
accept hit-rate as saturated and pivot to banking the confirmed
cost-axis wins (footprint, p99 tail) at fixed hit-rate.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

Acting on Cycle 0008's "hunt a new structure or bank cost wins" mandate,
the three variants split across surfaces: a deeper cost cut (sketch depth
4→2, v001), a genuinely new eviction structure (SLRU probation/protected
split, v002), and a microarchitectural increment swap (branchless
saturating add, v003). Every variant missed its primary claim — the two
cost probes regressed per-op-time and v001 also lost hit-rate; the SLRU
moved hit-rate only a sliver while charging footprint and per-hit work.
They are Winners only in the narrowest cost-axis sense.

## Patterns

- The increment loop body is confirmed a dead cost surface. Both edits
  that touched the per-row bump — removing rows (v001) and removing the
  branch (v003) — made the hot path *slower*, not faster. The saved work
  sits below noise, and on this saturating, well-predicted counter
  distribution the branchless rewrite forces an unconditional store where
  the branch let the common path skip it. Per-op gains must come from the
  eviction walk, not the sketch increment.
- Sketch depth is not actually accuracy-saturated at the bottom end.
  Cutting 4→2 leaked aliased estimates into admission and dropped
  hit-rate: the parent's extra rows were still doing real
  collision-correction. Depth=4 is the operating floor, not a saturated
  knob to trim.
- A real new eviction structure still hits the same ceiling. SLRU
  shielded twice-proven entries from one-hit newcomers exactly as
  intended, yet the existing `incoming_freq > victim_freq` admission gate
  already filters those newcomers — so list structure contributed almost
  nothing. The hit-rate ceiling is set by admission, not by list shape.

## Open questions

- With admission confirmed (again) as the binding constraint, is there
  any admission-side edit that isn't a margin/constant tweak — already
  ruled inert — that could move hit-rate?
- Are there *any* cost-axis savings left to bank that don't tax the hot
  path, now that depth, the increment loop, and list structure all carry
  a per-op or hit-rate cost?

## Continuation

Close three more threads: sketch depth has a hard floor at 4 (do not
trim it), the increment loop body is a confirmed non-surface for per-op
speed (neither fewer rows nor branchless bump helps), and SLRU/segmented
eviction is ruled flat because admission already does its job. Cycle 0008
already retired width, conservative update, and the admission margin.
The hit-rate-mover search is now exhausted across estimate, geometry,
admission-margin, and eviction-structure surfaces — treat hit-rate as
saturated at the incumbent base. Next cycle should either probe the one
untouched lever (the eviction *walk* / victim sampling, named by v003 as
the remaining cost surface) for tail/per-op savings, or stop hunting
hit-rate entirely and bank fixed-hit-rate cost wins.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
