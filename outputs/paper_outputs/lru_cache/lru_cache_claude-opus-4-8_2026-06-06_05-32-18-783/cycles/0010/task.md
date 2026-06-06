# Task: cycle-insight-writer

Cycle: 0010
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0010

## Winners

- v001
- v002
- v003

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0010/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0010/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0010/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0010/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0010/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0010/hypotheses/H03.md

## Prior cycles digest

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


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

This cycle attacked the one named, never-directly-tried hot-path cost —
per-hit link surgery — by replacing recency tracking with CLOCK
second-chance bits, from two placements, plus a footprint-shrink control.
The breakthrough was v003: a clean single-list CLOCK rewritten from
baseline cut both get and put latency, collapsed `Node` to 24 bytes by
dropping the doubly-linked recency list, and held hit-rate at/above exact
LRU. Grafting the same CLOCK bit onto the SLRU protected segment (v001)
regressed, and the struct-narrowing probe (v002) bought nothing — neither
advanced. v003 opens a fresh lineage that displaces the long-standing SLRU
champion.

## Patterns

- Placement, not the CLOCK mechanism, decided the outcome. Bolting a
  reference bit onto the SLRU protected segment (v001) backfired because
  the parent's protected-MRU branch was already write-free; CLOCK
  reintroduced a store on the hottest hits and taxed every link touch with
  mask/preserve. The same mechanism applied to a single flat list (v003),
  where there is no write-free MRU fast path to lose, turned every hit into
  one boolean write and won. The surgery the prior cycles flagged as
  "still holding room" was only being paid on the single-list structure.
- The headline cross-cycle assumption is now overturned: exact LRU recency
  tracking buys no hit-rate on this short-reuse trace. A single CLOCK bit
  matches LRU's hit-rate while removing the recency list entirely — cutting
  latency and per-node footprint together. The pointer surgery the
  segmented champion paid was pure overhead for this workload.
- The footprint-to-p50 coupling that haunted prior cycles is refuted as
  causal on the cache struct: shrinking the control block to `u32` fields
  (v002) left put flat and nudged get up. The earlier footprint
  perturbations were not a symmetric, exploitable lever on `LruCache`.

## Open questions

- v003 abandons segmentation wholesale — does the SLRU protected/probation
  split add anything over flat CLOCK on this trace, or was the entire
  segmented lineage carrying cost without payoff here?
- CLOCK's hit-rate edged above exact LRU; is that noise, or does the
  second-chance approximation genuinely suit this short-reuse pattern in a
  way strict recency does not?

## Continuation

Promote v003 (single-list FastHasher CLOCK, 24-byte node, no recency list)
as the new base — it is the first frontier mover in several cycles and
beats the SLRU champion on latency and footprint at equal hit-rate. Pursue
the flat-CLOCK lineage next: tune the clock (hand placement, sweep bound,
admission of fresh keys) rather than reaching back toward segmentation.
Drop CLOCK-on-SLRU (the protected fast path is already write-free) and
control-struct narrowing (footprint coupling is not a causal latency
lever). The standing "champion is at the frontier" conclusion no longer
holds — a structural change reached room the SLRU edits never could.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
