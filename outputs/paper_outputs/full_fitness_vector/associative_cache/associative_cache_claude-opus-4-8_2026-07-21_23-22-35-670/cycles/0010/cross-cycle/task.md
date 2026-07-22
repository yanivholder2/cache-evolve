# Task: cross-cycle-insight-writer

## Ledger (all cycles, cumulative)

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

Three baseline-derived variants each attacked a different fitness axis —
geometry (set-associativity), eviction policy (scan resistance), and item
metadata size — with no clean Winner emerging. Every hit-rate gain was
paid for with a per-op-time regression, and the only unambiguously
positive move was cutting dead metadata.

## Patterns

- Hit-rate and per-op-time moved as an antagonistic pair: both the wider-
  bucket geometry (v001) and the reuse-protection eviction tweak (v002)
  bought hit rate by lengthening a linear bucket/eviction scan, so neither
  is a free win.
- The bucket is scanned linearly, so anything that widens buckets or turns
  an O(1) eviction pick into a front-to-back scan directly taxes every
  operation — scan cost is the dominant hidden price in this cache.
- Metadata carried only by policies that are off the eval path (the
  `ClassicLRU` path never touches timestamps) is pure dead weight and can
  be cut with zero hit-rate cost, unlike the two hit-rate plays.
- Conflict-miss headroom at this working-set size appears thin: aggressively
  raising associativity barely moved hit rate.

## Open questions

- Can scan resistance be gained without converting eviction into a linear
  scan — e.g. maintaining the eviction candidate incrementally rather than
  re-scanning?
- Is there a bucket width that captures the collision headroom without the
  full scan-cost penalty, or is the trade strictly unfavorable here?
- Was v003's p50 drift real or measurement noise? Timing signals this
  cycle were small enough to warrant caution.

## Continuation

Pursue metadata trimming and other "cut dead weight off the eval path"
moves — they are cheap and monotonic. Treat both bucket-widening and
scan-based eviction protection as expensive: only revisit them with an
O(1)-maintained variant that avoids adding a linear scan. Drop plain
associativity increases at this working-set size. Be wary of small per-op
timing deltas; prefer changes with a clear structural reason to move.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle recombined the two cycle-0001 winners along disjoint axes and
probed a policy refinement. The Winners were the clean cross-breed (v001,
stacking scan-resistant reuse-protection onto the slimmed item layout) and
a deeper metadata trim (v003, peeling one more dead field off the eval
path). The policy refinement — trading permanent protection for
second-chance aging — was the sole loser.

## Patterns

- The cycle-0001 winners composed additively with no interaction: the
  hit-rate mechanism (a reuse flag) and the metadata mechanism (layout
  slimming) touch disjoint code, and the reuse signal rides on a field the
  slim layout already keeps, so stacking them cost no extra bytes.
- Metadata trimming stayed monotonic and cheap one field deeper — dead
  weight carried only by off-eval-path policies keeps peeling off with zero
  hit-rate change, only tail-latency noise.
- On this trace, permanent reuse-protection is the feature, not a bug:
  giving protection a self-clearing second chance sheds immunity too fast
  and collapses back toward plain LRU. Cold-but-once-hot "squatters" are
  not what caps hit rate here.

## Open questions

- How much further can metadata shrink before it collides with a field the
  eval path actually reads — is the frontier near its floor?
- Since second-chance aging failed, is there any way to bound protection
  (cap how long a hot item stays immune) without shedding it too fast, or
  is permanent binary protection simply the right fit for this trace?
- The recurring upward p99 tail drift under flat p50 keeps appearing on
  metadata cuts — structural or persistent measurement noise?

## Continuation

Keep stacking the two proven, disjoint mechanisms and keep trimming dead
metadata off the eval path — both remain cheap and additive. Drop
second-chance / CLOCK-style aging on this trace; permanent protection wins,
so refinements should preserve it rather than weaken it. Next cycle should
test where the metadata floor is and whether any hit-rate headroom remains
beyond the now-composed reuse-protection win.


### Cycle 0003

# Cycle 0003 Insight

## Cohort outcome

The cohort chased one prize from two directions — recover scan-resistant
hit-rate while holding the 16-byte metadata floor — plus a frequency-based
victim probe. Both floor-preserving paths landed as Winners: relocating the
reuse bit into the unmeasured bucket struct (v001) and buying scan
resistance from insertion geometry alone (v003). The LFU-tiebreak probe
(v002) was the sole loser.

## Patterns

- Scan resistance at the metadata floor is reachable two independent ways:
  explicit reuse state parked off the measured `Item` (v001) and pure
  probationary-front insertion with no per-item state (v003). Both hit
  hit-rate targets; the stateless geometric path was the clean one, while
  the stateful path recovered the same hit-rate but carried an
  implementation tax.
- The size metric weighs only `Item`, so state kept in the bucket is free
  on that axis — the exploit holds, and hit-rate and metadata-size stopped
  being antagonistic for these two Winners.
- The tax on v001 was implementation, not mechanism: a key-matched rebuild
  of the reuse bits on every hit is quadratic and blew out the tails, a cost
  an index-preserving swap would avoid.
- Frequency lost to recency again: evicting the least-frequently-reused item
  on saturated buckets was a worse victim signal than plain LRU-front,
  reaffirming that permanent binary protection fits this trace.
- Overlay hygiene bit v002: touching only the eviction file silently
  forfeited the parent's slim layout and regressed metadata — a win can be
  lost by omission, not just by logic.

## Open questions

- Can the bucket-bit path (v001) shed its quadratic reuse-bit rebuild via
  index-preserving swaps and then match or beat the stateless geometric
  path's clean tails?
- Does stateless probation (v003) leave any hit-rate on the table versus
  explicit reuse tracking, or is insertion geometry alone sufficient here?
- How to keep single-file variants from silently dropping prior-cycle wins
  carried in files they don't touch?

## Continuation

Treat v003's stateless probationary-front insertion as the cheap incumbent
to build on — it lifts hit-rate at the floor with better tails and no per-item
state. If pursuing v001's bucket-bit path, first replace the key-matched
rebuild with an index-preserving swap so the bits move for free. Drop
frequency/LFU-based victim choice; recency-front wins. Guard variants against
inheriting only part of the parent's layout.


### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

The cohort tried to push past the established hit-rate ceiling at the
16-byte floor from three angles: repairing the stateful reuse-bit path's
quadratic tax (v001), compounding explicit protection onto stateless
front-admission (v002), and widening the probation window via midpoint
insertion (v003). The two Winners each held the line rather than advancing
it — v001 recovered clean tails without moving hit-rate, v002 held the
metadata floor but gained nothing — while the midpoint geometry probe
regressed and lost.

## Patterns

- The index-preserving swap the prior cycle called for works exactly as
  predicted: mirroring each policy's single-element relocation onto the
  parallel reuse vector collapses the p99 tails and put p50 that the
  quadratic key-matched rebuild had inflated, with hit-rate held bit-for-bit
  identical. The parent's only real tax was the rebuild algorithm, not the
  scan-resistance mechanism.
- Explicit reuse protection and stateless front-admission are the same lever
  expressed twice, not compounding ones. Under move-to-back-on-hit, the
  frontmost slot is *always* the probationary one, so an added reuse bit
  resolves to the eviction index the position ordering already selects —
  v002 came out behaviorally indistinguishable from its parent. Protection
  adds value only over a policy that does not positionally segregate reused
  items.
- The admission-geometry axis has a sharp optimum at the front slot.
  Midpoint admission is strictly worse: the longer proving window it grants
  hot-but-unrehit items is dwarfed by the cost of letting never-reused scan
  items linger above the eviction slot, dropping hit-rate and nudging tails
  up. Front's value is precisely that a fresh unproven miss is the next
  thing evicted.
- The hit-rate ceiling held against every angle this cycle. Two independent
  attempts to exceed it at the floor produced no gain — evidence the current
  scan-resistance mechanisms are saturated, not that a better encoding of
  the same signal remains to be found.

## Open questions

- v001 flagged a metadata regression off the 16-byte floor that its own diff
  cannot explain (`Item` is still key+value; the reuse vector lives on the
  unmeasured bucket). Is this an overlay/inheritance artifact rather than a
  real layout change?
- If explicit protection and front-admission are redundant under the current
  policy, is there hit-rate headroom only reachable by changing the base
  policy's ordering itself — or is this trace's ceiling genuinely structural?
- Every stateless and stateful path now converges on the same hit-rate.
  Where does the remaining headroom, if any, actually live?

## Continuation

Adopt v001's index-preserving swap as the clean stateful incumbent — it
delivers the floor-level hit-rate with tails matching the stateless path.
Stop stacking explicit reuse protection onto front-admission: the two levers
are redundant under move-to-back-on-hit, so future protection ideas must
override a policy that does *not* positionally segregate reused items to
have any effect. Treat the front slot as the settled optimum on the
admission-geometry axis and drop midpoint/wider-probation variants. Next
cycle should first resolve v001's unexplained metadata regression (likely an
overlay artifact), then, since same-signal re-encodings have plateaued,
probe whether any hit-rate headroom requires changing the base policy
ordering rather than layering state on top of it.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

The cohort attacked the plateaued hit-rate ceiling exactly where cycle 4
pointed — the base policy's promotion ordering — from three angles:
gradual single-step promotion (v001), a two-hit reuse-bit probation gate
(v002), and a genuine two-segment SLRU bucket (v003). No variant qualified
as a Winner, but the cohort produced its first real evidence that hit-rate
headroom exists and is reachable only by adding true segment structure, not
by re-timing a single-list promotion.

## Patterns

- Re-timing promotion on a single list does not buy hit-rate on these
  short associative buckets. Slowing promotion (single-step, v001) actively
  regressed — a hot key gets evicted before it can ratchet out of the front
  zone — and gating promotion behind a reuse bit (v002) left the resident
  set unchanged, buying latency but not accuracy. Gradual/frequency-like
  promotion needs more slots than a small bucket provides.
- Genuine segment separation is the lever that moved hit-rate. Only v003,
  which gives twice-hit keys a reserved protected segment a one-hit scan
  burst cannot evict, crossed the front-admission ceiling. This confirms
  cycle 4's read that remaining headroom requires changing the base
  ordering itself, and identifies *segmentation* (2Q/SLRU-style) as the
  specific structural change that unlocks it.
- The hit-rate gain is bought with latency, not metadata. v003 keeps its
  segment boundary off-`Item` (16-byte floor holds), but every access now
  rotates and rebalances two segments, pushing get/put tails up. The new
  frontier is a hit-rate/latency trade, no longer a hit-rate/metadata one.
- The index-preserving swap keeps paying off (v002 collapsed the quadratic
  p99 tails as predicted), but the overlay-hygiene hazard recurred: v002
  carried only the bucket file and silently reverted `Item` off the 16-byte
  floor to the 40-byte base — a metadata regression by omission, not logic.

## Open questions

- Can v003's SLRU win keep its hit-rate gain while shedding the per-access
  rotate/rebalance latency — e.g. cheaper boundary bookkeeping or lazy
  demotion?
- What protected-segment cap is optimal for these short buckets, and does
  the gain survive at the smallest bucket sizes?
- How much headroom is left above the SLRU ceiling, or is two-segment
  separation itself near the structural limit for this trace?

## Continuation

Promote v003's segmented (SLRU) bucket to the incumbent worth building on —
it is the only path this cycle that broke the hit-rate ceiling, and it does
so at the metadata floor. Next cycle should focus on reclaiming its latency:
reduce the per-access rotate/rebalance work and tune the protected cap,
rather than chasing further hit-rate. Drop single-list promotion re-timing
(single-step under-protects; reuse-bit gating is redundant with position) —
the evidence is now firm that segmentation, not promotion timing, is the
lever. Carry v002's index-preserving swap into any stateful path, and
enforce full parent-layout inheritance so wins like the 16-byte `Item` are
not dropped by single-file overlays.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

The cohort worked to consolidate cycle 5's segmentation win rather than
chase new hit-rate: reclaim the metadata floor the SLRU incumbent had lost
by omission (v001), shed its per-access rotation latency (v002), and probe a
bounded protected cap to keep a live probationary zone (v003). The clean
win was v001 — the segmented bucket now delivers its ceiling-breaking
hit-rate at the 16-byte floor. v003 held the floor but its cap bet
regressed hit-rate; v002's lazy reorder recovered latency but leaked a hair
of hit-rate.

## Patterns

- The cycle-5 "40-byte" metadata cost of segmentation was never a cost of
  segmentation — it was pure overlay-hygiene debt. Trimming the dead
  `Item` fields the SLRU bucket never reads was behaviourally inert on
  hit-rate and reclaimed the full floor, and incidentally sped the
  byte-shifting put path. The recurring overlay hazard, once repaired,
  cost nothing.
- The per-hit protected rotation is confirmed pure overhead in the
  mixed-segment case — but it is not free to drop. It stops mattering only
  while a probationary item shields index 0; once a bucket saturates to
  all-protected, that same rotation was silently maintaining the intra-
  protected eviction order, so removing it trades eviction quality for the
  latency win. Latency reclamation here is entangled with the saturated-
  bucket edge case.
- The protected-cap knob is real but sharply mis-tuned at `max_size/2`.
  Capping that aggressively on short associative buckets demotes genuine
  reuse winners back to probationary and evicts them as scan-fodder,
  lowering hit-rate rather than lifting it. The "protection saturates to
  LRU" concern is genuine, but the corrective cap must sit close to
  `max_size` and demote only when protection is near-total.

## Open questions

- Can the protected rotation be shed for latency without the saturated-
  bucket hit-rate leak — e.g. lazy demotion that reorders only when the
  bucket is fully protected?
- Where does the optimal protected cap actually sit for these short
  buckets — near `max_size`, and does a near-total-only demotion trigger
  turn v003's regression into a gain?
- With the metadata floor now recovered on the segmented incumbent, is
  there any hit-rate headroom left above the SLRU ceiling, or is two-
  segment separation the structural limit for this trace?

## Continuation

Adopt v001 as the incumbent: it is the segmented (SLRU) bucket delivering
the ceiling-breaking hit-rate at the 16-byte floor, with the overlay-
hygiene debt finally cleared — build on it, not on cycle-5/v003. Do not
re-run the aggressive `max_size/2` cap; if pursuing a protected cap at all,
set it near `max_size` and demote only when protection is near-total, and
verify it against v003's regression. Treat v002's latency play as
conditional, not settled — the rotation may only be safely dropped when the
bucket is not fully protected, so any lazy-reorder retry must handle the
saturated case explicitly. Enforce full parent-layout inheritance on every
overlay so the reclaimed 16-byte `Item` is never dropped again.


### Cycle 0007

# Cycle 0007 Insight

## Cohort outcome

The cohort probed for hit-rate headroom *above* the recovered SLRU
incumbent from three directions — widening per-set associativity (v001),
giving the bucket cross-eviction ghost memory in 2Q style (v002), and
retuning the protected cap toward near-total protection (v003). None gained
on any dimension; the two Winners (v001, v002) qualified only by holding the
metadata floor while every hit-rate bet regressed. This cycle is a cluster
of falsifications: it maps three walls around the current ceiling rather
than breaking through it.

## Patterns

- Set *diversity* beats per-set *depth* at this size. Trading bucket count
  for bucket width (v001) gave each SLRU set more promotion slots but
  coarsened the hash spread, and hit-rate fell — so the ceiling is not a
  bucket-width limit. Widening also surfaced the predicted scan cost as a
  doubled `get` tail without moving the median.
- Cross-eviction memory does not lift hit-rate here. Admitting ghost-hit
  keys as pre-protected (v002) over-inflates the protected set: a
  re-reference is not proof of long-term heat, and blanket protection erodes
  the very reuse-bit discrimination that earned the incumbent its ceiling.
  It also taxed every `put` with linear ghost bookkeeping.
- The protected-cap knob has now been falsified at both extremes. Cycle 6
  ruled out `max_size/2` (too aggressive); v003 rules out `max_size - 1`
  (near-total protection degenerates the two-segment order into LRU, killing
  scan-resistance and lengthening protected-MRU rotations). The optimum sits
  strictly between the ends, not at either wall.
- Off-`Item` metadata hygiene held everywhere this cycle — ghost rings and
  cap knobs all stayed off the measured struct, so the 16-byte floor was
  never at risk. The overlay hazard that recurred in prior cycles did not
  reappear.

## Open questions

- If neither more associativity, more memory, nor more protection lifts
  hit-rate, is two-segment SLRU at diverse small buckets simply the
  structural ceiling for this trace?
- Where does the protected cap actually optimize, now that both extremes are
  falsified — is there a mid-range setting that turns a hold into a gain, or
  is the default already near-optimal?
- Is any remaining headroom a hit-rate question at all, or has the frontier
  fully shifted to reclaiming the incumbent's per-access rotation latency?

## Continuation

Treat the segmented incumbent's hit-rate as at or near its structural
ceiling for this trace: three independent headroom bets (wider
associativity, 2Q ghost memory, near-total protection cap) all regressed, so
stop chasing hit-rate gains through more depth, more memory, or more
protection. Keep the metadata floor and hash-spread diversity as settled —
do not trade bucket count for bucket width again. If the protected cap is
revisited, search the mid-range only; both extremes are now falsified. The
most promising unclaimed lever is latency: the per-access protected rotation
remains pure overhead outside the saturated-bucket case, so next cycle
should pursue lazy/conditional demotion to shed rotation cost while holding
the ceiling, rather than another hit-rate probe.


### Cycle 0008

# Cycle 0008 Insight

## Cohort outcome

The cohort turned fully onto the latency frontier that cycle 7 pointed to,
attacking the incumbent's per-access costs from three angles: skip the
below-cap protected rotation (v001), replace the O(n²) reuse-vector rebuild
with an index-preserving move (v002), and re-probe the protected cap at a
mid-range setting (v003). The real win was v002 — collapsing both p99 tails
while holding the best-on-record hit-rate and the 16-byte floor bit-for-bit.
v001 proved its target inert but unmeasurable; v003 confirmed the cap is a
nearly flat lever.

## Patterns

- The incumbent's blown-out tails were never intrinsic to carrying
  scan-resistance — they were a snapshot-and-rebuild artifact. Mirroring the
  policy's own single-element relocation onto the parallel reuse vector
  (v002), instead of reconstructing it by key-search each hit, makes the
  scan-resistance state free on latency as well as metadata. This is the
  cycle's clean, portable latency win.
- Not every per-access move is worth optimizing: the below-cap protected
  rotation is a single-element shift beneath the harness's 100ns timer
  resolution, so removing it (v001) is provably hit-rate-inert yet cannot
  register as a p50/p99 gain. Latency effort must target costly paths
  (per-hit rebuilds, front shifts), not sub-resolution single-element moves.
- The protected-cap knob is now mapped end to end and reads as nearly flat
  across its usable range. Both extremes were falsified in prior cycles; the
  mid-range `3*max_size/4` (v003) moves hit-rate only a hair and pays for it
  in a doubled `get` tail from deeper protected-MRU rotations. This lever is
  effectively exhausted as a hit-rate source.
- Metadata hygiene held throughout — every change stayed off the measured
  `Item`, and the anchored-parent discipline (v002 chose its parent
  explicitly to avoid the 40-byte overlay trap that sank the earlier attempt)
  kept the 16-byte floor intact.

## Open questions

- With the O(n²) rebuild gone, where do the incumbent's remaining tails sit —
  the `insert(0,…)`/`remove(0)` front shifts, or is the path now essentially
  at its latency floor?
- Is there any latency headroom left that the harness can actually resolve,
  or has sub-100ns granularity become the binding constraint on further
  micro-optimization?

## Continuation

Adopt v002's index-preserving reuse-vector move as the incumbent: it
delivers the record hit-rate with clean p99 tails at the 16-byte floor, and
supersedes the rebuild-based path. Stop tuning the protected cap — all three
regions (aggressive, near-total, mid-range) are now explored and it is a
flat lever; treat it as settled at the parent default. Do not pursue
sub-resolution single-element optimizations like v001's rotation skip; they
are correct but unrewardable by the eval harness. If latency is pursued
further, target the costlier front-shift paths (`insert(0)`/`remove(0)`) and
first confirm there is a harness-resolvable gain to capture, rather than
another hit-rate probe — the hit-rate ceiling remains firm.


### Cycle 0009

# Cycle 0009 Insight

## Cohort outcome

The cohort pushed on both remaining frontiers off cycle 8's index-preserving
incumbent: two latency bets (a cheap integer hash to shed SipHash setup —
v001; a scalar bitmask for the reuse state — v002) and one last hit-rate
probe (a two-hit admission threshold — v003). The clean win was v002 —
folding the parallel `Vec<bool>` into a single `u64` mask collapsed the last
per-hit heap shift and the eviction scan into register bit-ops, holding the
record hit-rate and 16-byte floor bit-for-bit. v001's hash swap was correct
but unresolvable; v003 regressed hit-rate.

## Patterns

- Latency is only capturable when the target lives on a heap-vector path,
  not on a fixed per-op constant. Folding the reuse state into a scalar
  bitmask (v002) removed a real O(bucket) shift and registered; swapping
  SipHash for a one-multiply mixer (v001) targeted a tens-of-nanosecond
  constant that sits below the harness's ~100ns timer, so it bought nothing
  measurable and even let coarse buckets push tails the wrong way. Hashing
  is not the observable lever on this harness.
- The in-residency protection threshold is now falsified as a hit-rate
  source. Demanding a second hit before granting protection (v003) does not
  filter enough scan noise to clear the ceiling — the same bar that rejects
  one-hit-wonders also delays protection for genuinely hot keys until a scan
  can evict them, so it regresses. One-hit binary protection sits at or above
  the sweet spot for this trace.
- Scan-resistance is now free on every dimension at once — metadata (proven
  cycle 8), and now latency (v002) — because under small associativity the
  whole reuse set fits in a scalar the CPU manipulates branch-free. The mask
  splice mirrors the policy's single-element relocation exactly, so
  representation changes stay behaviourally inert.

## Open questions

- With the reuse vector gone scalar, are the incumbent's remaining tails now
  purely the policy's own `items` front-shifts (`insert(0)`/`remove(0)`), and
  is there a harness-resolvable gain left there at all?
- Has the frontier now fully exhausted — hit-rate ceiling firm across every
  swept lever, latency down to items-vector shifts and timer granularity?

## Continuation

Adopt v002's `u64` reuse-mask as the incumbent: it delivers the record
hit-rate with the collapsed `put` p99 tail at the 16-byte floor, superseding
the `Vec<bool>` path. Stop probing hit-rate through in-residency protection
thresholds — one-hit is confirmed at/above the sweet spot and the two-hit bar
regresses; combined with prior cycles, the hit-rate ceiling should be treated
as firm. Do not chase the hash constant or any other fixed per-op cost below
the ~100ns timer — only heap-vector paths are resolvable. If latency is
pursued further, aim solely at the `items` front-shifts and first confirm a
harness-resolvable gain exists before spending a variant on it.


### Cycle 0010

# Cycle 0010 Insight

## Cohort outcome

The cohort ran one Pareto-composition bet and two headroom re-probes off the
scalar-mask incumbent: fold cycle 9's `u64` reuse-mask onto cycle 9's
integer-hash bucket (v001), re-open the untested *narrower*-sets geometry
direction (v002), and re-open protection in the one saturated regime where the
binary bit can no longer rank survivors (v003). The clean, decisive win was
v001 — it married the record hit-rate to the latency floor at the 16-byte
metadata floor, bit-for-bit. v002 and v003 both regressed hit-rate and only
qualified by holding the metadata floor; each closes a previously-open thread
as a falsification.

## Patterns

- Orthogonal levers compose cleanly, and that is where the frontier moved this
  cycle. The best hit-rate (hash-driven distribution) and the best latency
  (reuse-set representation) had lived in separate variants; because hash and
  reuse-set representation are independent, splicing them yielded a strict
  Pareto improvement with no trade-off — the mask splice mirrors the policy's
  single-element relocation exactly, so hit-rate held to the digit.
- The geometry seam is now falsified on *both* sides. Prior cycles ruled out
  wider buckets; v002 rules out narrower ones — quartering set width to
  quadruple set count raised conflict misses and widened the put tail. Per-set
  associativity, not set count, guards the SLRU ceiling; the harness default
  split sits at the optimum, not on a slope.
- Permanent one-hit protection is load-bearing even at saturation. The last
  untested weakening — clearing the mask only when a bucket is fully protected
  (v003) — still regressed: a saturated mask retains per-key reuse evidence
  that outlives one eviction, and zeroing it readmits stale/scan traffic. Every
  attempt to re-open protected slots, on any schedule or regime, now regresses.

## Open questions

- With hit-rate and latency now co-located at their floors, is there any
  harness-resolvable dimension left to move, or is the incumbent at the joint
  frontier for this trace?
- Are the remaining tails purely the policy's own `items` front-shifts, and is
  there a resolvable gain there at all?

## Continuation

Adopt v001 as the incumbent: it is the strict Pareto point — record hit-rate,
latency floor, 16-byte metadata floor, all at once — superseding both the
hash-only and mask-only parents. Treat three seams as now closed: the geometry
lever is falsified on both the wider and narrower sides (stop repartitioning
buckets-vs-slots), and protection weakening is falsified in every regime
including saturation (stop probing hit-rate by re-opening protected slots). The
hit-rate ceiling should be considered firm and fully mapped. If any lever
remains, it is the `items` front-shift latency path — but first confirm a
harness-resolvable gain exists before spending a variant, since the incumbent
already sits at the timer floor on the measured paths.


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
