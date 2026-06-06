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
