# Task: cycle-insight-writer

Cycle: 0005
Output dir: /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0005

## Winners

- v003

## Variant insights

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0005/variants/v001/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0005/variants/v002/insight.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0005/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0005/hypotheses/H01.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0005/hypotheses/H02.md
- /home/user/cache-evolve/outputs/associative_cache_2026-06-10_20-07-06-786/cycles/0005/hypotheses/H03.md

## Prior cycles digest

### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle stress-tested the prior Winner's frequency-first rule from three
angles: composing it with probationary front admission (v001), aging its
counters under eviction pressure (v002), and a clean-sheet segmented-LRU
built on `lfu_counter` as a promote-on-second-hit flag (v003). Only v003 won;
both modifications to the inherited frequency rule regressed below their parent.

## Patterns

- The two levers from Cycle 0001 do not compose — they conflict. Stacking
  front admission onto frequency-first eviction makes a fresh key
  simultaneously the recency-oldest and minimum-frequency entry, turning it
  into the immediate victim and discarding rising keys at probation.
- The winning scan-resistance came from explicit *segmentation* with a
  promote-on-second-hit rule, not from tuning a single frequency scalar.
  Demanding proof of reuse before granting protection beat both pure-LFU
  victim selection and single-position LRU admission.
- The dominant pressure on this workload is capacity collision, not
  staleness: aging the survivors' counters removed exactly the durable-count
  protection that made high-frequency keys valuable. "Once-hot immortality"
  was a feature here, not a failure mode.
- Both winning and losing variants still pay an O(bucket_size) linear victim
  scan; the per-op cost of that scan remains structurally present but largely
  unmeasured in the reported vectors.

## Open questions

- Can the linear front-scan for the first probationary item be made O(1)
  (e.g. a tracked segment boundary) without losing the SLRU hit-rate gain?
- Is there a probationary/protected sizing ratio that improves on the
  implicit split, or does promote-on-second-hit already capture most of it?
- Since aging hurt and immortality helped, is there any workload regime
  (shifting working set) where staleness *would* dominate — or is this trace
  firmly collision-bound?

## Continuation

Build on v003's segmented bucket as the new baseline: it is the strongest
scan-resistant admission scheme so far. Pursue cost-side refinements —
especially eliminating the linear probationary scan — and segment-sizing
tuning, while keeping hit-rate the primary axis. Drop both counter aging and
front-admission composition; this cycle showed they erode the protection that
durable frequency/segmentation provides on this workload.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle probed the inherited segmented-LRU (v003 of Cycle 0002) along
three axes: probationary eviction order (v002), equal-frequency tiebreak
granularity (v001), and a protected-segment cap with LRU demotion (v003).
The Winner, v002, reversed probation to evict the *oldest* zero-count entry —
making probation a FIFO queue — which recovered rising keys the prior
newest-first churn was discarding.

## Patterns

- Within the segmented bucket, *which* probationary item is evicted is itself
  a high-leverage lever: keeping the same scan but reversing it to oldest-first
  gives a fresh key a full lap to earn its second-hit promotion, and that
  alone lifted hit-rate with no change to admission, promotion, or metadata.
- Two of three changes were inert on hit-rate. The equal-frequency tiebreak
  (v001) and the protected cap (v003) left retention identical — true recency
  and the position proxy pick the same victims, and the protected segment
  never saturated, so its cap-and-demote branch never fired.
- The inert changes split on cost: v001's reorder-free path is a structural
  per-op *saving* (removes the per-hit shuffle), while v003's cap check is a
  structural per-op *cost* (extra linear pass per promotion) with no offset.
- Consistent with prior cycles, the trace remains collision-bound: the gain
  came from giving rising keys a promotion window, not from recency or
  staleness signals, which stayed neutral.

## Open questions

- Is the FIFO-probation win compatible with v001's reorder-free tiebreak — can
  the per-op saving be banked alongside the hit-rate gain?
- Does any workload regime ever saturate the protected segment, making v003's
  cap relevant, or is it permanently dead code on this trace?
- With probation order now tuned, what is the next hit-rate lever — segment
  sizing, or is the FIFO+promote-on-second-hit scheme near its ceiling here?

## Continuation

Adopt v002's oldest-first (FIFO) probation as the new baseline — it is the
strongest scan-resistant scheme so far. Pursue composing it with v001's
reorder-free tiebreak to capture per-op savings without risking hit-rate, and
push on segment-sizing as the remaining hit-rate axis. Drop the protected-cap
demotion: it is inert on this trace and only adds promotion-path scan cost.


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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
