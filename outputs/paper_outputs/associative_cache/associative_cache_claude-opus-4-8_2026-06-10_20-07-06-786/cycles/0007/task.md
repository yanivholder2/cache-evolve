# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0007

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0007/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0007/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0007/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0007/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0007/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle attacked the inherited FIFO-probation segmented-LRU from three
directions — a stricter two-hit promotion threshold (v001), a capped
protected segment with demotion-to-probation (v002), and a CLOCK
reference-bit substitute for recency ordering (v003). None beat the parent:
all three regressed hit-rate. v001 was carried as the Winner only as the
least-harmful change; the prior FIFO baseline effectively stands unimproved.

## Patterns

- Every variant lost by *withholding or stripping protection* from keys the
  parent kept. Raising the promotion bar (v001) stranded once-hit keys in
  FIFO until eviction; capping the protected segment (v002) demoted proven
  hot keys back into harm's way; CLOCK (v003) discarded the fine recency
  ranking that shielded warm keys. The trace punishes any scheme that makes
  earned protection harder to get or easier to lose.
- This confirms a multi-cycle boundary: one probationary hit is sufficient
  proof of reuse here, and the protected segment is load-bearing and does
  not want shrinking. Both the second-hit threshold (this cycle) and counter
  aging / front-admission (earlier cycles) failed for the same reason —
  they erode durable protection on a collision-bound trace.
- Recency granularity matters: a coarse one-bit reprieve underperforms full
  positional ranking in small buckets, so recency ordering is not safely
  approximated away.

## Open questions

- With promotion threshold, protected cap, and recency approximation all
  ruled out, is the FIFO-probation + promote-on-first-hit scheme at or near
  its hit-rate ceiling on this trace?
- Is there a *looser* protected sizing (larger, not capped) that helps, or
  is the current implicit split already optimal?
- Are the remaining gains now cost-side only (banking the per-op savings
  from reorder-free tiebreak) rather than hit-rate?

## Continuation

Keep Cycle 0003's FIFO-probation segmented-LRU as the baseline; this cycle
produced no hit-rate improvement on it. Stop probing protection-tightening
levers — second-hit thresholds, protected caps, and recency approximations
have now each failed for the same protection-erosion reason. Redirect the
next cycle toward (a) cost-side wins such as composing the reorder-free
tiebreak to bank per-op savings, and (b) untested hit-rate axes like looser
protected sizing or bucket geometry interactions, since the admission and
promotion logic appears saturated.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

All three variants built on the FIFO-probation segmented-LRU baseline and
attacked it from different angles: an LFU-min fallback for full-protection
eviction (v001), a ghost-ring accelerating re-entry of recently-evicted keys
straight into the protected segment (v002), and a cost-side fast path that
elides a no-op reorder on the hottest hit path (v003). v003 won — but only as
a hit-rate-neutral tie; both behavioral changes regressed below the parent,
so the baseline again stands hit-rate-unimproved.

## Patterns

- The two hit-rate attempts failed for the same reason that has recurred all
  run: both weakened the discriminating power of earned protection. The LFU
  fallback let `== 1` newcomers dominate the tally and killed freshly-promoted
  rising keys; the ghost ring let weakly-warranted bounce-back keys seize
  protected space without earning it. Recency at the protected end and
  earn-it-each-time probation both beat their proposed replacements.
- This sharpens the multi-cycle boundary: on this collision-bound trace,
  "recently evicted" is not a reliable proxy for "hot," and banked frequency
  counts do not separate hot from rising keys better than plain protected-LRU
  recency. The admission/promotion/eviction logic is now repeatedly confirmed
  saturated for hit-rate.
- The only non-regressing change was purely cost-side, and it landed as an
  undifferentiated tie because hit-rate is the sole fitness axis. A provably
  behavior-neutral per-op saving is invisible to selection — there is no
  dimension to reward it.

## Open questions

- Is hit-rate now effectively at its ceiling on this trace, given that
  admission, promotion, protected-eviction signal, and re-entry have all been
  ruled out across cycles?
- Without a per-op-time fitness dimension, can any cost-side refinement ever
  be selected for, or will it always tie? Is adding such an axis warranted?
- The one untested hit-rate axis remaining is looser/larger protected sizing
  and bucket geometry — does enlarging protection (rather than tightening or
  re-routing it) move the needle?

## Continuation

Keep the FIFO-probation segmented-LRU as the baseline; this cycle produced no
hit-rate gain. Stop probing protected-eviction re-signaling (LFU fallback) and
accelerated re-entry (ghost rings) — both join the long list of
protection-erosion failures. The behavioral logic looks saturated for
hit-rate. Redirect the next cycle to the two genuinely untested directions:
(a) looser/larger protected sizing and bucket-geometry interactions as the
last plausible hit-rate lever, and (b) recognizing that cost-side wins like
v003 are real but unrewardable under a hit-rate-only fitness — consider whether
a per-op-time axis is needed before spending more cohort slots on them.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

After several cycles of behavioral saturation, this cohort finally found a
hit-rate lever by leaving the bucket logic alone and changing *geometry*: v001
halved the bucket count and doubled bucket size (capacity conserved), winning
on associativity alone. The two behavioral variants both regressed — seating
promotions at the protected-LRU boundary (v002) and reviving two-hit promotion
inside wider buckets (v003) — so the segmented logic again stands unimproved
while the routing geometry moved the needle.

## Patterns

- The breakthrough came from outside the admission/promotion/eviction logic
  entirely. The trace was mildly associativity-starved: one bucket-count
  halving let each collision set hold more of the hot working set without
  eroding any earned protection, confirming geometry is the live axis that
  behavioral tuning is not.
- The two regressions reaffirm the multi-cycle protection-erosion boundary
  from a fresh angle. Seating promotions at the protected-LRU end (v002) made
  rising keys re-earn MRU standing and they got evicted sooner; the parent's
  promote-straight-to-MRU placement is better here.
- Wider buckets did not rehabilitate the known-bad two-hit threshold (v003):
  a longer probation lap still does not buy back keys that a stricter promotion
  bar strands. The two-hit regression is intrinsic to demanding a second proof
  of reuse on this trace, not an artifact of narrow baseline geometry.
- The win is described as small, suggesting the regime is near its
  associativity sweet spot — a second halving would likely trade routing away
  for diminishing returns.

## Open questions

- Is one halving the optimum, or is there a slightly different split (or a
  geometry that interacts with segment sizing) that extends the gain before
  routing loss dominates?
- Does the wider-bucket win change the cost picture — the larger per-bucket
  scan is invisible to hit-rate-only fitness but is a real per-op cost; is a
  time axis now worth adding?
- With geometry now confirmed live but behavioral logic saturated, are the two
  axes independent, or does the best segment policy shift once buckets widen?

## Continuation

Adopt v001's single-halving geometry on top of the FIFO-probation segmented-LRU
as the new baseline — this is the first hit-rate gain in several cycles and the
geometry axis is now the most promising direction. Probe geometry further but
cautiously: test whether the sweet spot is exactly one halving or admits finer
tuning, and whether segment sizing interacts with the wider buckets. Stop
reviving the two-hit threshold (now failed at both geometries) and stop
re-seating promotions below MRU — both join the long protection-erosion list.
The single-hit promote-to-MRU placement is settled; spend slots on geometry,
not promotion logic.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
