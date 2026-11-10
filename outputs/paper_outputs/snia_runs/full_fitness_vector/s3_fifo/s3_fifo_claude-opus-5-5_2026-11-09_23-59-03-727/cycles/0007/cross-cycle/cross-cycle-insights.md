# Cross-Cycle Insights

## Recurring wins

- **One change per variant.** Bundled changes in cycles 1–2 hid their causes and moved gains back and forth between hm_0 and web_0. Single-change variants (cycles 3 onward) produced the only real wins and made causes clear.
- **Judge credit by reuse inside main.** Reuse-gated main-victim ghosting and resetting freq/reused at graduation each helped on its own (cycle 3) and added up when combined (cycle 4). That combination is the only change so far that improved every trace/size point.
- **Spend ghost budget selectively, not generously.** Filters that free ghost slots lengthen the useful history at no cost. Selective filtering has beaten raising the bound every time.
- **More tenure for strongly reused keys.** Raising MAX_FREQ (cycle 5) helped hm_0 at the small size. Faster aging (cycle 7) hurt, which points the same way from the other side.
- **Timing gains come from structural changes that cut real work.** Slab + hash (cycle 2) and Fx hashing (cycle 3) are the only timing gains whose diffs explain them, and both kept hit rates bit-identical.

## Recurring failures

- **Resizing the small queue.** Fixed 10% (cycles 1 and 5) and 1/3 (cycle 6) both regressed. The 1/4 share is a local optimum.
- **Adaptive small/main split (ARC-style).** It regressed in cycle 1 and has not been retried.
- **Longer or larger ghost history.** Bound = capacity (cycle 4) let stale reuse displace live residents. Resizing or splitting ghost regions (cycles 1–2) only moved the hm_0/web_0 trade-off around.
- **Changing ghost→main admission in either direction.** Looser admission (re-admit with credit, cycle 6) and stricter admission (send back to small, cycle 7) both hurt web_0 at the large size.
- **Extra probation passes inside small.** Reinsertion of freq-1 keys (cycle 6) and the looser `freq >= 1` graduation threshold (cycle 2) duplicated the ghost path or let unreused keys into main.
- **Faster freq aging in main.** Halving on each sweep evicted high-freq keys whose hits are far apart.
- **Hash-table layout tweaks.** A merged index (cycle 4), 2x over-allocation (cycle 5) and a ghost ring with a liveness probe on every insert (cycle 7) all made the `put` tail worse. Memory footprint and probes per miss matter more than amortized cleanup.
- **Unbounded eviction work per miss.** Multi-pop loops and eviction cascades reliably inflate `put` p99.
- **Timing claims from policy-only diffs.** Every cycle showed timing drift its diff couldn't explain, so it is noise. Give no timing credit to policy-only diffs.

## Open questions

- Is web_0's large-size gap out of reach for the S3-FIFO family? Probation length, small share, ghost bound and both directions of ghost admission have all failed.
- What moves hm_0 at the small size? Candidates: protecting keys at the freq cap, a size-dependent small/main split or MAX_FREQ, or levers that act on every main eviction rather than only on ghost hits.
- Can the miss path drop a hash probe, for example with fingerprints, a denser ghost index, or ghost liveness tracked without a separate `ghost_index` lookup?
- Should metadata reporting add heap bytes per resident key? Per-entry size has stayed flat while ghost and index heap grew unseen.
- The benchmark environment is still not pinned. Same-machine baselines are needed before trusting small timing differences.

## Per-policy notes

- **S3-FIFO (sole policy so far):**
  - The current best parent is cycle-5 v002: Fx slab + reuse-gated ghosting + reset at graduation + MAX_FREQ 7, with a 1/4 small share, a `freq > 1` threshold and the ghost bound at `main_size`. Cycles 6 and 7 produced nothing better.
  - hm_0 responds to how long main retains keys. Gains at the large size look saturated, and what's left is a small-cache problem.
  - web_0 at the large size is the stubborn cell. Every probation and ghost-handoff lever tested so far has regressed it.
  - Per-entry metadata has stayed flat (24 B) in every cycle. New flag bits fit into padding or tag space.
