# Task: cycle-insight-writer

Cycle: 0009
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0009

## Winners

- v002
- v001
- v003

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0009/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0009/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0009/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0009/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0009/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0009/hypotheses/H03.md

## Prior cycles digest

### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle ran two latency micro-optimizations against the 24-byte champion
and one geometry-sizing probe. Both micro-opts — eliding arena bounds
checks via unchecked indexing (v001) and fusing the unlink+push_back
relink to drop "dead" sentinel writes (v002) — held hit-rate and node size
but regressed per-op latency, advancing nothing. The lone advance was v003:
nudging the protected soft cap from 4/5 to 7/8 of capacity recovered
hit-rate above the parent with no latency penalty.

## Patterns

- Both latency micro-opts failed for the same reason and it is a cost-model
  error: the "redundant work" each removed was already free. Bounds checks
  on sequentially-predicted, proven-in-bounds arena accesses were elided or
  cheap (v001); the eliminated NIL stores were store-buffer-absorbed writes
  to an already-resident cache line (v002). In both cases the change traded
  a non-cost for real codegen damage — unsafe risk and register/code-size
  pressure from inlining — and came out net slower. The hot-path latency of
  this SLRU lives in hashing, link surgery, and cache-line traversal, not in
  the checks or writes these variants targeted.
- The Cycle 0004 lesson that protected-cap widening costs latency is now
  shown to be magnitude-dependent, not categorical. v003's modest one-eighth
  step bought hit-rate for free — get/put p50 even ticked down — whereas
  v003-from-Cycle-0004's larger share starved the fast path. There is real
  headroom between the parent's 4/5 geometry and the point where latency
  breaks down.

## Open questions

- How far can the protected fraction be pushed past 7/8 before the
  fast-path starvation seen at larger shares reappears — is there a clean
  cap maximizing hit-rate while latency holds?
- With the obvious micro-opt levers tapped, is the 27 ns champion latency
  near this trace's floor, or does a structural change (hashing, traversal
  layout) still have room the redundant-work edits never reached?

## Continuation

Promote v003's 7/8 protected cap as the new base — it is a free hit-rate
gain and the only frontier mover this cycle. Next cycle, probe the
remaining geometry headroom (a fraction between 7/8 and the starvation
point) rather than reaching straight for the largest share. Drop
bounds-check elision and the relink fusion as tried-and-regressed: both
attacked non-bottlenecks. If pursuing latency further, target the actual
hot-path costs (hashing, link surgery, cache-line traversal), not
compiler-handled redundancy.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

This cycle probed the headroom around the new 7/8 protected-cap champion
from three angles: a larger geometry step to 15/16 (v001), a stricter
two-hit probation gate (v002), and a last-key get cache to skip the
HashMap probe (v003). Both surviving Variants held the 24-byte node but
regressed their bases — neither advanced the frontier. The lone latency
probe (v003) never produced a measurement, lost to a simulator infra
failure rather than on merit.

## Patterns

- The protected-cap headroom between 7/8 and the starvation cliff is now
  closed from the top. Cycle 0006 found free hit-rate at 7/8; stepping to
  15/16 confirmed a sharp starvation cliff in that gap — the near-full
  protected segment leaves the fast path no room, so get/put p50 and p99
  all jumped for a small hit-rate gain. 7/8 sits just below the breakdown
  point, and there is little usable geometry left above it.
- Admission cleverness keeps failing for the same workload reason. As in
  Cycle 0005's two-hit gate, the two-hit warm filter drains short-reuse
  keys out of probation before their second hit lands, so it discards
  genuine repeaters and drops hit-rate — while adding per-hit bookkeeping
  that nudges get p50 up. The free-bit mechanism worked; the policy is
  the recurring loser on this short-reuse trace.
- The mechanism-vs-policy split holds yet again: 15/16 sizing and the
  warm-bit packing both kept the 24-byte layout byte-for-byte; every loss
  came from behavior (geometry starvation, admission policy), not from the
  representation changes.

## Open questions

- With both the top of the geometry headroom and admission gating now
  exhausted, is the 7/8 champion effectively at this trace's
  hit-rate/latency frontier?
- Does the H03 last-key cache — the only attempt that actually targeted
  the named get hot-path cost (HashMap hash+probe) — have merit? It was
  never evaluated, so the latency-via-traversal/hashing thread remains
  genuinely untested, not refuted.

## Continuation

Keep Cycle 0006's 7/8 protected cap as the standing base; nothing this
cycle displaced it. Drop the 15/16 geometry step (starvation cliff
confirmed) and the two-hit warm gate (admission gating is tried-and-
regressed on this short-reuse workload — stop revisiting it). The one
live thread worth re-running is H03's last-key get cache: it attacks the
real hot-path cost and died on infra, not measurement, so re-issue it
before concluding the champion is at the frontier. If pursuing latency
further, stay on actual hot-path costs (hashing, link surgery, cache-line
traversal), not geometry or admission policy.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

This cycle spent all three slots closing the live threads the prior
cycles left open: two probe-skipping get caches (v001 single last-key,
v002 four-slot hot-key side table) attacking the named HashMap-probe
hot-path cost, and one smarter ghost re-admission (v003) testing whether
the gated-ghost hit-rate gain reproduces. All three held the 24-byte node
but none advanced — the surviving Variants regressed their base. The
Cycle 0006 7/8 protected-cap champion stands undisplaced.

## Patterns

- The HashMap probe is now settled, from two angles, as not the get
  bottleneck. The baseline lookup is a zero-sized xor-multiply
  `FastHasher` plus an arena index probe — already cheaper than anything
  layered in front to skip it. The single-key guard chain (Option-compare
  plus a key clone) and the four-slot linear scan each added more work to
  the common path than they removed, and both more than offset their
  intended saving. The H03 last-key thread, finally measured here after
  dying on infra, is refuted on merit.
- The cross-cycle gated-ghost question is now answered negatively.
  Reuse-distance re-admission — treating one cross-eviction return as
  proof of hotness and admitting straight to protected — floods the
  protected segment and evicts genuinely hot keys, costing both hit-rate
  and latency. The Cycle 0004 hit-rate record does not reproduce on this
  short-reuse trace; it was trace/size-contingent.
- A recurring unscored cost surfaced: every variant kept `Node` at 24
  bytes by parking its state on `LruCache`, but that grew the cache struct
  (96→120/168 bytes) and the footprint alone perturbed put p50 even when
  put was byte-identical — locality damage the scored metadata-size never
  captures.

## Open questions

- With probe-skipping and ghost admission both now refuted, is the 7/8
  champion at this trace's hit-rate/latency frontier, or does the one
  untouched hot-path cost — link surgery and cache-line traversal — still
  hold room?
- Does growing the `LruCache` struct itself carry a real latency tax
  worth treating as a constraint, given how consistently footprint growth
  alone moved p50?

## Continuation

Keep Cycle 0006's 7/8 protected cap as the standing base; nothing this
cycle moved it. Three threads are now closed — stop revisiting them: any
structure that skips the HashMap probe (the probe is not the bottleneck),
ghost/reuse-distance re-admission (does not reproduce here), and admission
gating generally. The only named hot-path cost never directly attacked is
link surgery and cache-line traversal during promote/demote — if pursuing
latency, target that, and prefer changes that do not grow the `LruCache`
footprint. Absent a fresh structural lever, treat the champion as at or
near this trace's frontier.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
