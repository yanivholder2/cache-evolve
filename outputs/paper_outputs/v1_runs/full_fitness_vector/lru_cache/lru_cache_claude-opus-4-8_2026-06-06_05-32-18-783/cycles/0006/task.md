# Task: cycle-insight-writer

Cycle: 0006
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0006

## Winners

- v003
- v002
- v001

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0006/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0006/variants/v002/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0006/variants/v003/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0006/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0006/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0006/hypotheses/H03.md

## Prior cycles digest

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


## Write

Exactly one file: `cycle-insight.md` in the output dir.
