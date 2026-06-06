# Task: cycle-insight-writer

Cycle: 0004
Output dir: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0004

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0004/variants/v001/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0004/variants/v002/insight.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0004/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0004/hypotheses/H01.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0004/hypotheses/H02.md
- /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0004/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
