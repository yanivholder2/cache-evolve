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
