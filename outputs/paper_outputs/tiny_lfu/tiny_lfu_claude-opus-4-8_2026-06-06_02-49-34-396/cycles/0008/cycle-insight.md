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
