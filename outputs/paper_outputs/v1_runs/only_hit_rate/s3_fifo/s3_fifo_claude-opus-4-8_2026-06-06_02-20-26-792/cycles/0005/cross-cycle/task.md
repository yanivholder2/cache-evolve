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
