# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants forked from baseline, each a single-axis lever:
segmented-LRU for hit-rate (v001), narrow `u32` sentinel links for
metadata-size (v002), and a fast inline integer hasher for per-op-time
(v003). The hasher swap won — the only change that improved a hot-path
dimension without paying a hard regression elsewhere.

## Patterns

- Per-op-time was the contested axis: both losing variants moved their
  target dimension but bought it with added per-op latency (extra
  promote/demote branching in v001, per-access index-width casts in v002).
- The Winner improved per-op-time directly and even picked up a free
  metadata side-win (dropping the default hasher's seed) — cheap,
  near-pure changes on the hot path beat changes that trade one axis
  against another.
- Representation/policy changes that widen the node or sprinkle
  conversions did not pay for themselves in locality.

## Open questions

- The SLRU hit-rate gain (v001) is real and targets the LIRS workload's
  weakness — can it be made cheaper so it stops trading against latency?
- v002 flagged that narrow links could be kept without per-access casts;
  is there a cast-free encoding that lands the metadata win?
- v003's median win came with a get tail-latency regression — is the
  hasher change strictly worth it, or does the tail need attention?

## Continuation

Build on v003 as the per-op-time base. Pursue the narrow-link
metadata win again but without per-access width casts, and probe whether
the SLRU hit-rate lever can be reintroduced without its latency cost.
Drop the naive `u32`-cast-everywhere approach as tried-and-regressed.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle revisited the two open levers from Cycle 0001 and probed a
hot-path skip. The standout Winner combined the SLRU hit-rate policy
with the integer hasher and segment-aware fast paths, defending the
record hit-rate while pulling latency back to hot-path-class — proving
the policy and latency levers are orthogonal and stackable. The
narrow-`u32`-link metadata re-encoding also landed as a Winner, hitting
the smaller node, while a standalone MRU-guard branch lost.

## Patterns

- Hit-rate (segmentation policy) and per-op-time (cheap hasher + skipping
  no-op list surgery) are independent axes: the cycle's best Variant
  optimized both at once with neither paying for the other.
- Skipping redundant list surgery only pays when the skip condition is
  cheaply predictable. Conditioned on segment position (already-MRU within
  protected) it helped; as a bare data-dependent `tail == idx` branch on
  the universal hot path it mispredicted and regressed both median and
  tail — the same idea won in one framing and lost in another.
- The narrow-link metadata win is genuinely achievable and centralising
  the casts keeps the slowdown modest, but re-encoding `Option` niches as
  explicit sentinels still costs measurable median and tail latency — a
  "pure representation" change is not actually latency-free.

## Open questions

- Can the metadata (24-byte node) and the latency-optimal hot path coexist
  in one Variant, or does any sentinel encoding inherently tax per-op-time?
- The winning fast path skipped surgery using a predictable position
  check — are there other no-op-touch cases (same-segment, head) skippable
  the same cheap way without inviting a mispredicting branch?

## Continuation

Carry forward the combined SLRU-plus-fast-hasher Variant as the new base —
it holds both the hit-rate record and competitive latency. Next cycle
should try folding the narrow-`u32`-link metadata win onto that base and
measure whether the encoding's latency tax survives once the fast paths
absorb it. Drop bare data-dependent MRU-guard branches on the universal
hot path; only pursue surgery-skips gated on cheaply predictable position
conditions.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

This cycle pushed against the two stuck records — the pinned hit-rate and
the unfolded metadata win — and re-tested a hot-path skip. The metadata
fold-onto-SLRU attempt never produced a result (an infra/disk-quota
failure, not a design verdict). The ghost-history hit-rate attempt lost,
and the surviving Winner — grafting the surgery-skip onto compact
single-list LRU — landed only marginally, holding a metadata+latency
Pareto point rather than scoring a real latency gain.

## Patterns

- The surgery-skip fast path is fundamentally segment-local: it won inside
  the SLRU protected segment (where the touched node is reliably the MRU)
  but on single-list plain LRU the touched node is rarely the tail, so the
  guard mispredicts and adds put/tail cost for a ~negligible get-median
  gain — the same lesson as Cycle 0002's bare-branch case, now confirmed
  to generalize: the position gate pays only where the predicted position
  is actually common.
- Reuse-distance re-admission has the right intuition for the LIRS trace
  but unconditional promotion straight into protected overshoots:
  transient post-eviction re-references evict genuinely hot protected keys,
  dropping hit-rate *below* pure SLRU while still paying ghost-bookkeeping
  latency (a HashMap op) on every put and worsening locality with a larger
  struct.

## Open questions

- Does a gentler ghost admission — re-admit into probation, or gate on
  repeated ghost hits rather than a single one — recover the reuse-distance
  signal without over-promoting and breaking hit-rate?
- The narrow-link metadata fold onto the SLRU-plus-fast-hasher base remains
  genuinely untested (its variant died on infra, not merit): can the
  24-byte-class node and the latency-optimal hot path coexist?

## Continuation

Re-run the metadata fold onto the Cycle 0002 SLRU base — it failed for
disk-quota reasons, not design, and is still the most promising open
lever. If pursuing hit-rate via ghost history, make admission gentle
(probation re-entry or repeated-hit gating); drop unconditional
ghost-to-protected. Drop the surgery-skip on single-list plain LRU
entirely — keep it only segment-gated. The Cycle 0002 combined SLRU base
remains the champion; nothing this cycle displaced it.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle finally cashed in two long-open levers and saw them both land
cleanly. The metadata fold (v001) re-encoded the SLRU champion's recency
links as `u32` sentinels and packed the segment flag into the link's high
bit, crossing into the 24-byte node class while holding the hit-rate
record and even shaving latency. The gentle ghost-admission rework (v002)
gated re-admission on repeated evidence and routed single hits to
probation, recovering the reuse-distance signal that overshot in Cycle
0003 and pushing hit-rate past the record — also without a latency tax.
Both were Winners. The raw protected-cap widening (v003) lifted hit-rate
but paid per-op latency, and lost.

## Patterns

- The two stuck records — the unfolded metadata win and the
  reuse-distance hit-rate signal — both finally yielded, and notably with
  *no* paying axis: tightening the layout repaid itself in locality, and
  gentler ghost admission sped the hot path by avoiding protected churn.
  Pure-representation and policy-gating changes can be genuinely free when
  they reduce work rather than add a branch.
- Hit-rate has two distinct levers on this LIRS trace: *retention* (how
  much working set protected holds) and *admission* (what earns a
  protected slot). v002's gated admission gained hit-rate for free, while
  v003's blunt 90/10 cap widening gained the same axis but starved the
  protected-MRU fast path — confirming once more that moving the
  segment geometry trades hot-path latency, whereas smarter admission does
  not.
- The metadata fold that died on infra in Cycle 0003 was sound on merit:
  packing the flag into `prev` (not a separate `bool`) is exactly what
  turns a 32-byte result into a 24-byte one, and centralized cast helpers
  avoided the per-access width tax that sank earlier attempts.

## Open questions

- v001 and v002 are independent wins on different axes (metadata vs.
  hit-rate) — can the 24-byte node and the gated ghost admission be folded
  onto one base, the way SLRU and the fast hasher stacked in Cycle 0002?
- Is there more retention headroom from admission cleverness (e.g. tuning
  the repeated-hit threshold) now that blunt cap-widening is shown to cost
  latency?

## Continuation

Promote the v001 24-byte node and the v002 gated-ghost admission as the
new bases, and next cycle attempt to combine them into a single Variant
holding both the smaller node and the new hit-rate record — they touch
disjoint structures (node layout vs. external ghost store), so the fold
should be clean. Keep pursuing hit-rate through admission gating, not
through raw protected-cap widening; drop the blunt 80→90% split as
tried-and-regressed. The Cycle 0002 SLRU base is now superseded on both
the metadata and hit-rate axes.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle attempted to cash the Cycle 0004 follow-ups: fold the gated
ghost admission onto the 24-byte node (v001), tighten admission with a
two-hit probation gate (v002), and fuse the promote+demote pair to cut
hot-path churn (v003). All three held the 24-byte node class and all
three landed net neutral-to-negative on their bases — none advanced the
frontier. The standing champion (Cycle 0004's 24-byte node) was not
displaced.

## Patterns

- Every "free mechanism" claim held but every policy it encoded moved
  the wrong way. Spare-high-bit flags (`REF_MASK`/`PROT_MASK`), the
  in-struct ghost store, and pure control-flow refactors each added zero
  node bytes — yet the gated ghost fold, the two-hit gate, and the fusion
  all regressed latency and/or hit-rate. The size mechanism is proven
  cheap; the behavioral lever is what failed.
- Workload-a's reuse is short-distance, and that drains the admission
  levers. Ghost admission fired too rarely to move recall; the second-hit
  gate delayed promotion of genuinely-hot keys and pushed hit-rate below
  parent. The hit-rate headroom that admission cleverness opened in
  Cycle 0004 did not reproduce here — it looks trace/size-contingent, not
  a general win.
- "Fusion" of two correct list-surgery calls is not redundancy removal:
  the demote-on-overflow path already does exactly four surgeries, so
  collapsing the calls performs the same work plus a hot-path branch and
  a parallel code path — regressing even the untouched `put`, the tell of
  codegen perturbation rather than algorithmic saving.

## Open questions

- Is the Cycle 0004 gated-ghost hit-rate gain actually realizable on
  workload-a, or was it specific to the trace/size it was measured on?
  v001's near-zero recall delta is a direct warning that the fold's
  premise may not hold here.
- With admission gating now tapped out on this workload, where does
  remaining headroom live — back on the latency/layout axis, or is the
  current champion close to this trace's ceiling?

## Continuation

Drop the v001 ghost-onto-24-byte fold: it is structurally per-op-costly
while its benefit is absent on workload-a — the cross-cycle additive
prediction did not hold. Drop two-hit admission gating and the
promote/demote fusion as tried-and-regressed. Stop chasing hit-rate via
admission cleverness on this workload; before any further policy work,
the next cycle should verify whether the Cycle 0004 hit-rate record even
reproduces on workload-a, and otherwise pivot back to latency/layout
levers. Cycle 0004's 24-byte node remains the standing base.


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

Exactly one file: `cross-cycle-insights.md` in the output dir. Use the following four fixed sections — preserve names and order:

## Recurring wins

- <pattern that has held up across multiple cycles>

## Recurring failures

- <approach that keeps not paying off, and why>

## Open questions

- <thread worth probing in upcoming cycles>

## Per-policy notes

- <observations grouped by eviction policy>
