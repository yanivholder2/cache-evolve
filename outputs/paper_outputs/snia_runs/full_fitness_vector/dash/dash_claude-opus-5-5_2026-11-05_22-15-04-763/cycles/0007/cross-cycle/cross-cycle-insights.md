# Cross-Cycle Insights (through cycle 0007)

## Recurring wins

- **Keep proven items resident.** The two-hit promotion gate, MRU demotion into stash, probing overflow and the probing-room gate each raised hit rate a little on every trace and size, and the gains stacked (0001–0004). The current lineage is gate + MRU demotion + probing overflow + probing-room gate (0004 v002).
- **MRU demotion for every demoted item works with or without the gate** (0002, 0003). Variants that weakened it lost hit rate (0005 v003).
- **Fewer cascades help only if stash keeps draining.** Probing overflow cut cascades and kept items moving out of stash, which recovered `put` p99 (0003). The single bounded deferral did the same (0005 v002).
- **Making the stash probe cheaper is the only lever that has moved `get` p50.** The stash presence mask (0007 v002, the first Winner) lowered p50 with bit-identical hit rate.
- **Cheap per-item state** (reusing the `lfu_counter` field) and **hashing once per lookup** improve things without growing metadata or changing hit rate. They are easy to stack onto other changes.

## Recurring failures

- **Skipping or deferring a cascade only moves its cost.** Unbounded deferral backed up the stash (0004). Bounded deferral pushed the cost onto a later `get` (0005). Skipping elsewhere pushed it onto `put` or onto probing lookups (0004).
- **Gating deferral on the state of one stash bucket fails.** Fill level is always near full (0006), and the marked-LRU signal (0007) just shifted cost between `get` and `put`. Neither separated a harmful deferral from a harmless one.
- **Changes to the common hit path other than the stash probe have done nothing for p50.** This covers lookup order (stash last made things worse, 0003), scan direction (0005), skipping the MRU reorder (0006), cascade avoidance (0004) and repeated hashing (0007).
- **Extra work on stash churn shows up on the hot path.** Examples are mask recompute on removal (0007), stash pressure from longer residency (0002, 0005) and the fused rotate at a second call site (0006).
- **The ungated MRU + overflow line** lost on `get` p99 because of the probing lookup penalty (0004). **Liveness signals taken from a single normal-tier stay** lost hit rate (0005).
- **Small p99 moves on paths a change doesn't touch** keep appearing (0005–0007). Treat them as possible noise or codegen effects until a rerun confirms them.

## Open questions

- Can the stash mask be maintained in O(1) on removal (counting bits, or bits stored next to bucket metadata) and keep the p50 gain without the `put` and `get` p99 cost?
- How much of the mask's tail cost comes from false positives and how much from refresh? Would a wider fingerprint help?
- Once the stash probe is cheap, does the Dash-level segment hash hold any of the remaining p50 gap?
- Can the fused in-place rotate, limited to the third-hit path or kept out-of-line, keep its `get` p99 gain without the `put` regression?
- Is there a deferral signal other than stash-bucket state that stops the small-cache hit-rate loss?

## Per-policy notes

- **Dash (only policy studied so far):**
  - hm_0 at the large size is the most sensitive cell for retention. It gains most from changes that keep items longer and loses most when retention drops. Use it as the first check for hit-rate regressions.
  - Cache size changes whether stash residency helps: longer stash stays help the large cache and hurt the small cache.
  - Stash hits are a large share of all hits, so checking stash first is the right order. The goal is to make that check cheaper, not to move it.
  - Base candidates: 0004 v002 for p50 and cost work, 0005 v002 for deferral and tail work, 0007 v002 (stash mask) as the base for the next round of mask-maintenance changes.
