# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-11-05T23:25:22+00:00

## timeline

### 23:25:46 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Items that have proven themselves are retained longer, and this has kept paying off since 0001. The levers are the two-hit gate, MRU demotion into stash, probing overflow, and the probing room gate (\"don't evict a proven resident\"). Hit rate has gone up a little each cycle in this lineage.\n- hm_0 at the large size is the most reliable signal for retention. It reacts most strongly to every gain and every loss (0001, 0002, 0003, 0005), so use it as the first check on any retention change.\n- When an approach cuts cascades and also keeps items flowing out of stash, the tail recovers. This held for probing overflow in 0003 and for the bounded single deferral in 0005.\n- Cost-only changes that leave hit rate bit-identical are the cleanest tests. They decided the scan-direction question (0005), the MRU-shift question (0006), the hash-count question (0007) and the stash-probe question (0007).\n- The `get` p50 gap came from the common hit path, as 0004/0005 suspected. It moved only after two fixes: a dense stash presence mask that skips the stash scan (0007), and a cheaper hash function (0008, the first Winner with clean improvement across all percentiles).\n- Cheap per-item state, such as reusing the `lfu_counter` field, adds gating at no metadata cost.\n\n## Recurring failures\n\n- Skipping or deferring a cascade only moves its cost: to `put` through stash or target fullness, or to a later `get` (0004, 0005, 0007). Unbounded deferral fills stash and was the worst case.\n- Deferral gates driven by signals local to the stash bucket did not work. A fill-level gate almost never fires at steady state (0006). The marked-LRU signal only shifted cost between `put` and `get` (0007). The deferral line is close to exhausted with this family of signals.\n- Several p50 levers have been ruled out: lookup order (checking stash last is worse because stash hits are common), scan direction, the near-MRU reorder skip, and the number of hash calls (0003, 0005, 0006, 0007).\n- Liveness signals taken from a single normal-tier stay hurt hit rate everywhere (0005). Every demoted item should go to stash MRU.\n- The ungated MRU and overflow branch never recovered baseline latency. Items placed in probing pay lookup and eviction costs (0003, 0004).\n- Any work placed on stash churn or on the item shift during promotion shows up directly on the hot path. Examples are full-bucket re-hash on mask refresh (0007) and fingerprint upkeep in the shift (0008).\n- Two read-path designs were worse than a dense mask word: a coarse or shrunken mask, where false positives rose, and an in-bucket fingerprint byte scan (0008).\n- Small p99 moves on paths a change does not touch keep appearing (0005 to 0008). Treat them as noise or codegen/inlining effects until a rerun confirms them.\n\n## Open questions\n\n- With the cheap hasher in place, does O(1) mask maintenance at 64-slot or finer resolution recover the rest of the `put` cost without hurting the read reject path?\n- Is there more p50 to gain in the segment-level hash/selection, or in the remaining mask false positives (a wider fingerprint)?\n- The `get` tail sits on the stash-hit promotion shift. Can that shift be made cheaper without adding work to it? Is the fused rotate (0006) worth retrying if it is restricted to the third-hit path or kept out of inlining?\n- Does the cheap hasher change the trade-offs on the deferral line (the 0005 v002 lineage), since part of that line's cost came from re-hashing?\n- What new kind of deferral signal could avoid the small-cache hit-rate loss? It would have to work beyond the state of a single stash bucket.\n\n## Per-policy notes\n\n- Dash (the only policy explored so far): the current best line is a stack built on the baseline, in this order: two-hit gate, MRU demotion, probing overflow, probing room gate, stash presence mask, fast hasher (0008 v003).\n- Dash, 0004 v002 to 0005 v002 line: the single deferral trades `put` p99 against `get` p99 and small-cache hit rate. A large cache gains from longer stash residency, and a small cache loses stash slots.\n- Dash, cost structure: `get` p50 depends on the stash probe and on hashing, and `get` p99 depends on the stash-hit promotion shift. `put` depends on stash pressure and on mask upkeep.\n- Infrastructure: the 0001 build failures (HTTP 429 from Maven/Gradle) said nothing about the policies involved. H01 and H03 were later covered as MRU demotion and probing overflow.\n"
}
```

### 23:25:53 thinking



### 23:25:53 chat

I've written `cross-cycle-insights.md` in the cycle 0008 cross-cycle folder. It has the four required sections in order, built only from the ledger for cycles 0001–0008 and with no numbers invented.

- **Recurring wins:** The approach that kept paying off is keeping items that have already been hit twice around longer, through the two-hit gate, demoting into stash at the most-recent end, and the probing changes. The `get` median only came down once the stash check got cheaper, first with the presence mask in 0007 and then with the fast hasher in 0008. The fast hasher is the first variant to improve every latency percentile without hurting hit rate. hm_0 at the large size is the cell to watch for any retention change.
- **Recurring failures:** Skipping or deferring a cascade only moved its cost from `put` to a later `get` or the other way round. Deferral gates based on the state of a single stash bucket didn't work. Several suspects for the `get` median gap were ruled out: lookup order, scan direction, the move-to-MRU shift and the number of hash calls. Any extra work on stash churn or on the promotion shift showed up on the hot path. Small p99 changes on paths a change doesn't touch should be treated as noise until a rerun confirms them.
- **Open questions:** Whether mask updates can be made cheap now that hashing is cheap. Whether more median latency is left in segment-level hashing or in mask false positives. How to make the promotion shift cheaper for the `get` tail. Whether the cheap hasher changes the deferral trade-offs, and what new kind of signal deferral would need.
- **Per-policy notes:** Dash is the only policy explored so far. The notes record the current best stack (ending in 0008 v003), the trade-offs of the deferral line, which operation drives which latency number, and that the 0001 build failures came from Maven/Gradle rate limits rather than the policies.


---

- finished: 2026-11-05T23:25:54+00:00
- duration: 31.49s
- status: ok
- input_tokens: 4
- output_tokens: 2361
- cache_read_tokens: 41579
- cache_write_tokens: 13339
- cost_usd: 0.163135
