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
