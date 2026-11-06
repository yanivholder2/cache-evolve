## Recurring wins

- Items that have proven themselves are retained longer, and this has kept paying off since 0001. The levers are the two-hit gate, MRU demotion into stash, probing overflow, and the probing room gate ("don't evict a proven resident"). Hit rate has gone up a little each cycle in this lineage.
- hm_0 at the large size is the most reliable signal for retention. It reacts most strongly to every gain and every loss (0001, 0002, 0003, 0005), so use it as the first check on any retention change.
- When an approach cuts cascades and also keeps items flowing out of stash, the tail recovers. This held for probing overflow in 0003 and for the bounded single deferral in 0005.
- Cost-only changes that leave hit rate bit-identical are the cleanest tests. They decided the scan-direction question (0005), the MRU-shift question (0006), the hash-count question (0007) and the stash-probe question (0007).
- The `get` p50 gap came from the common hit path, as 0004/0005 suspected. It moved only after two fixes: a dense stash presence mask that skips the stash scan (0007), and a cheaper hash function (0008, the first Winner with clean improvement across all percentiles).
- Cheap per-item state, such as reusing the `lfu_counter` field, adds gating at no metadata cost.

## Recurring failures

- Skipping or deferring a cascade only moves its cost: to `put` through stash or target fullness, or to a later `get` (0004, 0005, 0007). Unbounded deferral fills stash and was the worst case.
- Deferral gates driven by signals local to the stash bucket did not work. A fill-level gate almost never fires at steady state (0006). The marked-LRU signal only shifted cost between `put` and `get` (0007). The deferral line is close to exhausted with this family of signals.
- Several p50 levers have been ruled out: lookup order (checking stash last is worse because stash hits are common), scan direction, the near-MRU reorder skip, and the number of hash calls (0003, 0005, 0006, 0007).
- Liveness signals taken from a single normal-tier stay hurt hit rate everywhere (0005). Every demoted item should go to stash MRU.
- The ungated MRU and overflow branch never recovered baseline latency. Items placed in probing pay lookup and eviction costs (0003, 0004).
- Any work placed on stash churn or on the item shift during promotion shows up directly on the hot path. Examples are full-bucket re-hash on mask refresh (0007) and fingerprint upkeep in the shift (0008).
- Two read-path designs were worse than a dense mask word: a coarse or shrunken mask, where false positives rose, and an in-bucket fingerprint byte scan (0008).
- Small p99 moves on paths a change does not touch keep appearing (0005 to 0008). Treat them as noise or codegen/inlining effects until a rerun confirms them.

## Open questions

- With the cheap hasher in place, does O(1) mask maintenance at 64-slot or finer resolution recover the rest of the `put` cost without hurting the read reject path?
- Is there more p50 to gain in the segment-level hash/selection, or in the remaining mask false positives (a wider fingerprint)?
- The `get` tail sits on the stash-hit promotion shift. Can that shift be made cheaper without adding work to it? Is the fused rotate (0006) worth retrying if it is restricted to the third-hit path or kept out of inlining?
- Does the cheap hasher change the trade-offs on the deferral line (the 0005 v002 lineage), since part of that line's cost came from re-hashing?
- What new kind of deferral signal could avoid the small-cache hit-rate loss? It would have to work beyond the state of a single stash bucket.

## Per-policy notes

- Dash (the only policy explored so far): the current best line is a stack built on the baseline, in this order: two-hit gate, MRU demotion, probing overflow, probing room gate, stash presence mask, fast hasher (0008 v003).
- Dash, 0004 v002 to 0005 v002 line: the single deferral trades `put` p99 against `get` p99 and small-cache hit rate. A large cache gains from longer stash residency, and a small cache loses stash slots.
- Dash, cost structure: `get` p50 depends on the stash probe and on hashing, and `get` p99 depends on the stash-hit promotion shift. `put` depends on stash pressure and on mask upkeep.
- Infrastructure: the 0001 build failures (HTTP 429 from Maven/Gradle) said nothing about the policies involved. H01 and H03 were later covered as MRU demotion and probing overflow.
