# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0007

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

With the partition ratio frozen at ~4/96, this cycle abandoned main-queue
sizing and probed three genuinely orthogonal mechanisms off the converged
parent: ghost *sizing* alone (H01), warm-starting re-admitted entries'
frequency (H02), and switching main from pure FIFO to move-to-front recency
(H03). Only the ordering-discipline change (v003) landed in the Winners, and
only by a sliver; both ghost-side levers regressed.

## Patterns

- The two ghost-side bets failed for the same root reason: a lone ghost hit
  is weak reuse evidence, yet the unguarded direct-to-main path treats every
  ghost hit as confirmed. Deepening the ghost (v001) manufactured false
  re-admissions from ancient keys; warm-starting frequency (v002) let
  single-hit keys outlast a real eviction pass. Both crowded hotter
  residents. The discriminating signal is accumulated `get()` frequency, not
  the ghost hit itself — so neither ghost depth nor re-admission protection
  is a live lever here.
- The first lever to pay anything since sizing converged was the *ordering
  discipline* of main — making the resident queue recency-aware instead of
  position-blind FIFO. This confirms the LIRS trace carries recency structure
  the partition knob never touched. But the gain was marginal: the existing
  `freq` second-chance counter was already a decent recency proxy, so explicit
  move-to-front recovers only a sliver.
- The cross-cycle pattern holds: every lever that adds protection or memory to
  weak-signal keys regresses, while levers that sharpen which residents the
  eviction scan reaches can pay. Reordering (what gets evicted) beats
  re-admitting or warm-starting (what gets kept).

## Open questions

- The move-to-front win is thin — is there more in the ordering axis (e.g.
  reordering on promotion, or recency-aware tail selection) or is this lever
  also near its floor?
- With ghost sizing, ghost re-admission, frequency warm-start, and now
  ordering all probed, is the remaining headroom on this workload small enough
  to declare the configuration converged?

## Continuation

Keep the partition ratio frozen and the direct-to-main re-admission path
intact. Drop both ghost-side axes for good — ghost *sizing* without a gate is
inert-to-harmful, and frequency warm-starting on re-admission regressed;
neither deserves another probe. The one open thread is main's ordering
discipline: v003's move-to-front paid, barely, so the next cycle should test
whether the recency axis has more to give (alternative reorder triggers or
recency-aware eviction selection) before concluding the configuration is
converged.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

Off the converged ~4/96, move-to-front parent (cycle 0004 v003), this cycle
spent all three probes on main's ordering and eviction discipline:
recency-preserving in-place second-chance (H01), lowest-`freq` victim
selection within a tail window (H02), and tail-insertion of re-admitted and
promoted entries (H03). Every variant regressed; v001 and v002 "won" only as
the least-bad of an all-negative cohort. Nothing improved on the parent.

## Patterns

- The parent's `evict_main` rotation — `pop_back` plus `push_front` of
  surviving `freq > 0` entries — turned out to be load-bearing, not waste.
  Both H01 and H02 removed that rotation in favor of "in-place" or "more
  accurate" victim selection, and both regressed: the rotation was
  *refreshing* survivor recency and reinforcing v003's move-to-front
  ordering. Sharpening which victim the scan picks actively cost hit-rate
  because it discarded that refresh.
- The assumption that head-MRU purity is good and the tail is reliably
  coldest does not hold: `evict_small` and `put` also feed the head, so
  walking inward and decrementing every survivor (H01) over-erodes the `freq`
  signal of genuinely hot keys. The recency-vs-frequency tension is delicate
  and the parent already sits near its balance.
- H03 reconfirms the standing cross-cycle finding from the other direction:
  a ghost re-admission and a multi-hit promotion are strong-enough signals to
  deserve front protection on entry. Demoting them to the eviction-facing
  tail dropped them pre-reuse. The front position the parent grants these
  keys is earned, not waste.

## Open questions

- With both the victim-selection refinements and the insertion-position
  refinements regressing, is any remaining lever on the ordering axis live,
  or has v003's move-to-front already captured what the recency structure
  offers?
- Given that cycles 0003–0005 produced only sliver gains and now a fully
  negative cohort, is this configuration converged for practical purposes?

## Continuation

Treat the ordering axis as exhausted: the parent's rotation-on-eviction and
its front-insertion of re-admitted/promoted keys are both load-bearing — do
not disturb either. Every refinement this cycle (in-place decrement, windowed
victim selection, tail insertion) regressed, so drop all three. The
configuration appears converged; the next cycle should either declare it so
or find a lever on a mechanism not yet touched, not revisit eviction
ordering, victim sharpening, or insertion position.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
