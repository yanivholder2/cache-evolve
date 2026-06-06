# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0009

## Winners

- v001

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0009/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0009/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0009/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0009/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

Off the converged ~4/96, move-to-front parent, this cycle finally left the
ordering axis and probed three new mechanisms: eager (vs lazy) small-to-main
promotion timing (H01), ghosting main-evicted victims so cooled ex-residents
re-admit direct-to-main (H02), and a periodic global frequency-aging sweep
(H03). H02 (v002) landed in the Winners — the first genuinely new lever to
pay since sizing converged. H01 was inert; H03 regressed.

## Patterns

- The standing cross-cycle finding flips sign for the first time: *which
  population gets a ghost record* is a live lever, where ghost *depth* and
  ghost *gating* never were. Every prior ghost probe widened or gated history
  over the weak small-evicted one-hit-wonders and failed. Ghosting the
  main-evicted population — keys that earned promotion at least once, a
  strictly stronger reuse signal — recovered a signal the parent silently
  dropped. The discriminating axis is the *quality* of the ghosted key, not
  the size of the window.
- Promotion *timing* is confirmed inert (v001). At this thin small-queue size
  the admission FIFO does not churn fast enough to drop a freshly re-hit key
  before its accumulated `freq` is read at eviction, so eager and lazy
  promotion select the identical resident set. This echoes cycle 0003's
  finding that the partition knob absorbs distinct-on-paper levers — sizing
  has already fixed the outcome here.
- Time-based global aging is too blunt (v003). Uniform halving ages hot and
  stale residents alike, and since `freq` rebuilds only one bump per
  reaccess, the sweep evicts still-hot keys before they recover. The parent's
  eviction-time decay during the `evict_main` rotation already ages the cold
  tail precisely; a global sweep over-decays. This reconfirms the recurring
  lesson that the parent's rotation is load-bearing.

## Open questions

- v002's gain was slim because few main residents cool to `freq == 0` and
  then get reaccessed inside the unchanged `main_size`-bounded ghost window.
  Now that the *source* population is proven valuable, is there headroom in
  pairing it with ghost depth — a lever previously inert over the weak
  population but possibly live over this stronger one?
- Is the ghost queue worth splitting or weighting by source (main-evicted vs
  small-evicted), or does a single shared window suffice?

## Continuation

Adopt v002's main-victim ghosting as the new parent — it is the first real
gain in three cycles and reopens the ghost mechanism from a new angle. Drop
promotion-timing (inert under the thin partition) and global aging
(over-decays; the rotation already does targeted decay) for good. The live
thread for the next cycle is the ghost queue reconsidered over the *stronger*
main-evicted population: revisit ghost depth or source-aware ghost handling,
which prior cycles only ever tested over the weak small-evicted keys. Do not
revisit ordering, victim sharpening, or insertion position.


### Cycle 0007

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


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

Off the cycle-0007 source-aware-ghost-retention parent, this cycle finally
left the now-exhausted ghost-capacity axis and probed three new surfaces: a
runtime-adaptive small/main split steered by the ghost `from_main` tag (H01),
dropping weak one-hit-wonder ghosts entirely to hand the full budget to strong
keys (H02), and recency-aware move-to-front in the thin small admission queue
(H03). Only the adaptive split (v001) landed in the Winners — the first
mechanism in many cycles to *beat* static sizing rather than collapse back into
it. H02 was a severe regression; H03 was inert.

## Patterns

- Adaptivity is a genuinely new lever and the standout result. Every prior
  sizing result came from static sweeps that converged on the frozen ~4/96
  ratio; reusing the already-computed `from_main` steering signal to nudge the
  eviction *threshold* (no entries moved) turns the ghost queue into a feedback
  controller that tracks the workload's shifting balance. The gain is small —
  the tight clamp keeps the orbit near the proven center and this trace wanders
  only modestly — but it moved the right direction and revealed the static
  split was slightly main-starved.
- The weak small-evicted ghost population, repeatedly called "low value,"
  is in fact load-bearing. H02 confirms from the extreme that those
  `from_main: false` records are the *dominant* re-admission source: deleting
  them collapses hit-rate by roughly a fifth. This sharply qualifies the
  cross-cycle finding — source-aware *trimming* (0007) pays precisely because
  it still ghosts weak keys; *not ghosting them at all* starves the far larger
  weak-reuse population. Reallocating budget to strong keys does not relax
  their constraint.
- The small queue's intra-ordering is too short a lever to matter. At ~4%
  capacity move-to-front only changes an outcome in the narrow tail-with-refill
  window, and it works against itself by delaying the `freq > 1` promotion into
  main, so the two effects roughly cancel. The binding constraint stays in main
  and ghost capacity, not small-queue recency.

## Open questions

- v001's adaptive gain is bounded by a deliberately tight clamp around the
  proven center. Is there headroom in widening the orbit, changing the step
  size, or using a smarter steering signal than a fixed ±1 — or does this
  trace's modest phase structure cap the payoff regardless?
- With both weak-ghost removal (H02) and small-queue ordering (H03) falsified,
  is adaptivity the last live axis, or does the feedback-controller framing
  open other runtime-adaptive knobs (e.g. adaptive ghost capacity)?

## Continuation

Adopt v001's source-steered adaptive partition as the new parent — the first
lever in several cycles to beat static sizing instead of collapsing into it.
Drop weak-ghost removal permanently (severe regression; weak `from_main: false`
records are the dominant re-admission source and must keep being ghosted) and
drop small-queue move-to-front permanently (inert; the queue is too thin and it
delays promotion). The live thread for the next cycle is the adaptivity
mechanism itself — tune the orbit/step or extend the feedback-controller idea
to another runtime knob. Do not revisit static sizing, ghost depth, ghost
warm-start, weak-ghost removal, eviction ordering, victim sharpening, or
insertion position — all now exhausted or falsified.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
