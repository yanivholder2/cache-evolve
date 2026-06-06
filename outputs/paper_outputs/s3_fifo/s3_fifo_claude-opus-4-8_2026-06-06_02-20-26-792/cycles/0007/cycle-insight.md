# Cycle 0007 Insight

## Cohort outcome

All three probes ran off the cycle-0006 main-victim-ghosting parent and
reopened the ghost mechanism from the new stronger-population angle: ghost
*depth* now that history holds strong-reuse keys (H01), source-aware *retention
priority* — drop weak small-evicted records before strong main-evicted ones
within the same budget (H02), and a one-pass *warm-start* on main-evicted
re-admission (H03). Only the retention-priority change (v002) landed in the
Winners, and only at the margin; depth stayed inert and the warm-start
regressed.

## Patterns

- The cycle-0006 insight predicted the stronger population might wake the
  previously-dead depth lever. It did not. A main-evicted key that cooled to
  `freq == 0` is reaccessed inside the original `main_size` window or not at
  all, so entries retained past that bound never convert. Ghost *depth* is now
  falsified over both the weak and the strong population — the cycle-0006 win
  was the ghost *source*, never the window length. Retire depth for good.
- The one live refinement was *which source gets dropped first* under a fixed
  budget: sacrificing weak one-hit-wonder history before strong main-evicted
  history lets a few more proven keys survive to re-admit. This is a real,
  valid lever — but the gain is tiny because the weak-vs-strong eviction order
  only binds when the ghost is full *and* a displaced strong key would have
  been reaccessed before its weak neighbor expired, a narrow window on this
  trace. The deeper constraint remains raw ghost capacity, which depth cannot
  productively enlarge.
- Frequency warm-start regressed again, now for the third time and over the
  strong population. A one-pass survival credit only pays when the protected
  key is reaccessed inside that pass, and too few re-admitted main victims are.
  Origin-aware ghosting is a lever; layering eviction-protection on top of it
  is wasted slack, not recovered reuse. The cross-cycle finding holds firm:
  protecting/memorizing weak-signal keys regresses; sharpening retention
  priority pays, barely.

## Open questions

- v002's retention-priority gain is thin and bounded by raw ghost capacity,
  not source order. With depth dead, is there any remaining way to relax that
  capacity constraint without leaking one-hit-wonders, or is the ghost
  mechanism now fully characterized?
- Three cycles of ghost probing have yielded one slim source-quality win
  (0006) and one marginal retention-priority win (0007). Is the ghost axis now
  exhausted the way ordering was after cycle 0005?

## Continuation

Adopt v002's source-aware ghost retention as the new parent. Drop ghost
*depth* permanently — it is now falsified over both populations — and drop
re-admission warm-start permanently (regressed a third time). The ghost
mechanism looks close to exhausted: source *quality* (0006) and source
*retention priority* (0007) both paid only at the margin, and capacity, not
ordering or protection, is the binding constraint. The next cycle should
either declare the configuration converged or find a lever on a mechanism not
yet touched — not revisit ghost depth, ghost warm-start, eviction ordering,
victim sharpening, or insertion position, all of which are now exhausted.
