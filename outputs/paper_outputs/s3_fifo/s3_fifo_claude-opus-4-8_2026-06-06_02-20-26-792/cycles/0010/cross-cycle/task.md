# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

This cycle probed three independent S3-FIFO levers against the baseline —
admission threshold (H01), small/main partition ratio (H02), and ghost-queue
history depth (H03). Only the partition-ratio change (v002) landed in the
Winners; loosening the promotion gate and deepening ghost history both hurt
hit-rate.

## Patterns

- The three "move toward the paper's defaults" hypotheses split sharply: the
  sizing default helped, while the paper's promote-on-any-reuse threshold and a
  wider history window both regressed. On this LIRS workload the baseline's
  stricter filtering is load-bearing, not an oversight.
- Anything that admits more keys into the main resident queue — looser
  promotion (v001) or long-gap direct re-admission via deeper ghost (v003) —
  displaced hotter residents and cost more than it saved. The main queue is
  contention-sensitive; protecting it beats feeding it.
- Conversely, giving the main queue *more capacity* (v002) was the one net win,
  but only modestly, suggesting the partition ratio is a real but bounded knob.

## Open questions

- How far can the small/main ratio be pushed before the shrinking admission
  filter starts leaking one-hit-wonders into ghost/main? v002's gain was small —
  is there a sharper optimum past ~10/90?
- v003 showed history depth and the direct-to-main re-admission rule are
  coupled. Would deeper ghost history help if ghost hits were gated (e.g.
  re-admitted via small queue, or only when fresh) rather than pushed straight
  to main?

## Continuation

Pursue the partition-ratio lever further — sweep small/main beyond 10/90 to
find where the main-queue gain reverses. Drop standalone threshold-loosening
(H01-style) and naive ghost-deepening (H03-style); both regressed because they
crowd the main queue. If revisiting ghost depth, pair it with a gate on the
direct-to-main re-admission path rather than widening history alone.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle pushed three retention levers off cycle-0001's winning 10/90
parent: shrinking the admission queue further (H01), deepening the
frequency clamp (H02), and gating ghost re-admission through the small
queue (H03). Only the further partition shrink (v001, ~6/94) landed in the
Winners; deepening the second-chance ceiling barely moved, and throttling
ghost re-admission regressed.

## Patterns

- The partition ratio remains the dominant lever, but it is now visibly
  near its plateau: pushing from 10/90 to ~6/94 still paid, yet the gain
  shrank relative to the prior step. The optimum is close and the next
  shrink risks leaking one-hit-wonders rather than helping.
- Levers that target the *same* retention behavior as the partition ratio
  (raising the frequency ceiling) buy almost nothing once main is already
  large — hot keys clear the eviction gate well before the old cap bound
  them. Distinct-on-paper knobs collapse into the partition knob.
- Cycle 0001's "every direct-to-main path is contention-sensitive" finding
  does NOT generalize to the ghost re-admission path. A ghost hit is itself
  reuse evidence; demoting it to a small-queue re-confirmation discards
  that signal whenever the long-gap reaccess outlives small residency.
  Throttling here cost more than it saved.

## Open questions

- Where exactly does the partition ratio reverse? v001 is past the knee but
  still positive — one more probe (e.g. ~4/96) would locate the turn.
- With main near its useful maximum, are there levers *orthogonal* to main
  retention still untapped — e.g. ghost queue sizing/freshness, or the
  promotion gate threshold itself — rather than more ways to enlarge or
  protect main?

## Continuation

The partition-ratio sweep is nearly exhausted: take at most one more shrink
step to confirm the reversal, then freeze it. Drop frequency-ceiling
deepening (redundant with a large main) and ghost re-admission gating
(regressed — keep the direct-to-main re-admission path intact). Next cycle
should turn to levers orthogonal to main-queue retention, since stacking
more retention onto an already-large main is yielding diminishing returns.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle ran the partition-ratio sweep to its end (H01, ~4/96) and probed
two orthogonal levers off the ~6/94 parent: a freshness-gated deeper ghost
queue (H02) and a stricter small-to-main promotion gate (H03). Only the
final partition shrink (v001) landed in the Winners, and only by a sliver;
the freshness gate regressed and the stricter promotion gate was completely
inert.

## Patterns

- The partition-ratio sweep has effectively flattened. v001 still edged the
  parent at ~4/96, but the gain was a fraction of the prior step's — the knee
  is here. The workload tolerates a very thin admission filter without
  leaking one-hit-wonders, but there is no more meaningful retention to
  extract by enlarging main.
- Shrinking main's admission filter this far collapsed the promotion-gate
  lever entirely: with so small a small-queue, entries are evicted FIFO
  before they can sit at the tail as borderline two-hit keys, so `freq > 1`
  vs `freq > 2` is a no-op. The partition knob has absorbed the promotion
  knob — distinct-on-paper levers keep collapsing into sizing.
- The "every ghost hit is strong reuse evidence, send it direct-to-main"
  finding now holds across all three tried gating axes (small-queue
  re-admission, throttling, and position/freshness). The untried middle path
  from cycles 0001–0002 also failed: splitting ghost hits by recency
  discards a confirmed signal as fast as the deeper window earns it.

## Open questions

- With partition sizing, promotion threshold, and ghost re-admission gating
  all exhausted, what lever is genuinely orthogonal to main-queue retention?
  Candidates not yet meaningfully moved: ghost *sizing* on its own (without a
  re-admission gate), or the second-chance/frequency mechanics themselves.
- Is the remaining headroom on this workload small enough that the current
  ~4/96 configuration should simply be frozen as the operating point?

## Continuation

Freeze the partition ratio at ~4/96 — the sweep is done and the promotion
gate is now inert beneath it, so drop both. Abandon all ghost re-admission
gating (freshness, throttling, small-queue routing all regressed); the
direct-to-main path is correct and every variant on this axis has failed.
The next cycle should stop tuning main-queue retention entirely and seek a
lever on a different mechanism, or accept the configuration as converged.


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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

All three probes ran off the cycle-0008 source-steered adaptive-split parent
and stayed on the new adaptivity axis, refining the controller three ways:
momentum + a wider clamp so the split accelerates and roams further (H01),
hysteresis gating that only moves after agreeing evidence accumulates (H02),
and a second independent controller that self-tunes ghost *capacity* by its
measured conversion productivity (H03). Only the momentum/wider-clamp variant
(v001) landed in the Winners; hysteresis was a marginal regression and the
adaptive-ghost controller regressed harder.

## Patterns

- The controller was under-reaching, not mis-aimed. Letting the split
  accelerate on sustained agreement and roam a wider band recovered hits the
  tight single-step orbit could not reach — confirming the cycle-0008 suspicion
  directly. But the gain stays small because this trace's imbalance drifts only
  modestly and monotonically: most of the widened headroom and the higher step
  cap go unused, so momentum mostly buys *faster convergence within a narrow
  effective range*, not a new operating point.
- The per-event steering "thrash" the parent suffered was largely real
  directional signal, not noise. Hysteresis (H02) throttled a controller
  already pointed the right way, adding lag without sharpening, and lost hits.
  This trace's slow, near-monotone imbalance wants a *lower* commitment
  threshold, not a wider accumulation band — denoising discipline is the wrong
  medicine when the signal isn't noisy.
- The feedback-controller framing does *not* extend cleanly to ghost capacity
  (H03). Pinning ghost capacity near `main_size` beats letting a loop trade it
  away, because the controller's floor undercut the parent's static ghost
  budget and a transient dip in windowed conversion rate discarded exactly the
  re-admission history the change meant to capture. Ghost history here is
  hit-productive enough that it should stay pinned, echoing the standing finding
  that ghost capacity is the binding, must-protect constraint.

## Open questions

- v001's momentum gain is real but capped by the trace's modest, monotone
  drift. Is there any remaining adaptivity headroom, or is the partition
  controller now effectively tuned for this workload?
- Two cycles of adaptivity (0008 split, 0009 momentum) have each paid only at
  the margin, and the second runtime knob (ghost capacity) regressed. Is the
  adaptivity axis now near exhaustion the way ordering and ghost-capacity were?

## Continuation

Adopt v001's momentum + wider-clamp adaptive split as the new parent — a small
but real gain that confirms the controller was under-reaching. Drop steering
hysteresis permanently (this trace's imbalance is monotone signal, not noise;
gating only adds lag) and drop adaptive ghost capacity permanently (regressed;
ghost capacity must stay pinned near `main_size` — its floor leaks productive
history). The adaptivity axis is looking close to tuned-out: the partition
controller now tracks this trace about as well as its modest drift allows, and
the one untried runtime knob backfired. The next cycle should either declare
convergence or find a lever on a mechanism not yet touched — not revisit static
sizing, ghost depth/capacity, ghost warm-start, weak-ghost removal, steering
hysteresis, eviction ordering, victim sharpening, or insertion position, all now
exhausted or falsified.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

Off the cycle-0009 momentum adaptive-split parent, this cycle probed two
fresh surfaces — the small admission queue's intra-ordering (eager
`MAX_FREQ` promotion in H01, move-to-front in H02) and a freshness-weighted
controller step keyed to ghost-match depth (H03). Only small-queue
move-to-front (v002) landed in the Winners, and only barely; eager promotion
was inert and freshness-weighting regressed.

## Patterns

- The proven recency-reordering discipline finally paid on the one queue it
  had never touched. Applying main's move-to-front to the small hit branch
  makes `evict_small` shed the least-recently-accessed candidate instead of
  the oldest-inserted one, converting a short tail of re-touched keys. But the
  gain is tiny and the lever is near-saturated: small is too small and churns
  too fast for most candidates to earn a second hit, so the strict `freq > 1`
  promotion gate, not insertion order, still sets the filter's ceiling. Note
  this softly revises the cycle-0008 read that small-queue move-to-front was
  inert — under the now-adaptive split it nudges positive, but only at the
  margin.
- Promotion *timing* is not a live lever. Advancing a key into main the moment
  it saturates `freq`, rather than at eviction, changed nothing: a `MAX_FREQ`
  key already clears the promotion test and lands in main at eviction anyway,
  so moving only the moment neither admits new keys nor alters residency. The
  binding constraint is which keys promote and how long they reside, not when.
- Ghost-match depth carries no steering signal beyond its `from_main`
  direction. Jumping straight to the step cap on a "fresh" front-of-ghost hit
  made the split move faster but not better, landing below the parent —
  overshoot, not sharpening. This reinforces cycle-0009: the momentum
  controller is already well-aimed and converges fast enough; modulating step
  magnitude by signal recency is a dead end on this trace.

## Open questions

- v002's small-queue ordering win is real but explicitly near-saturated and
  bounded by small's size and promotion gate. Is there any remaining headroom
  on the small filter, or is it now fully characterized?
- Three mechanism families (split adaptivity, ghost capacity, small-queue
  ordering/timing) have each now paid at most marginally or regressed. Is the
  configuration effectively converged for this workload?

## Continuation

Adopt v002's recency-aware (move-to-front) small admission queue as the new
parent — a small but real gain, the first time the ordering lever helped on
the small queue under the adaptive split. Drop eager `MAX_FREQ` promotion
permanently (pure timing, inert — promotion already happens at eviction) and
drop freshness-weighted controller step permanently (overshoots; ghost depth
carries no steering signal beyond `from_main`). Every probed axis is now at or
past saturation: split adaptivity, ghost depth/capacity/warm-start, weak-ghost
removal, small-queue ordering and promotion timing, eviction ordering, victim
sharpening, and insertion position are all exhausted or falsified. The next
cycle should seriously weigh declaring convergence; if it continues, it must
find a mechanism not yet touched rather than re-tune any of the above.


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
