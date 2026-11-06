# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-11-05T23:03:18+00:00

## timeline

### 23:03:36 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-11-05_22-15-04-763/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-cycle insights (cycles 0001–0006)\n\n## Recurring wins\n\n- **Keeping proven items resident raises hit rate every time.** The two-hit gate (0001), MRU demotion to stash, both gated (0002) and ungated (0003), probing overflow (0002, 0003) and the probing room gate (0004) each gained hit rate on every cell tested, or close to it. Every one of them keeps re-referenced items alive longer.\n- **hm_0 at the large size is the cell that shows retention effects.** In every cycle, changes that help retention gain most there, and changes that hurt retention lose most there. Use it as the first place to check any retention change.\n- **Probing overflow cut the cascades that `put` p99 pays for, while still letting items leave stash.** It is the only cascade reduction that improved tails without moving the cost to another path (0003).\n- **Small, cheap per-item state is affordable.** Reusing `lfu_counter` added no metadata.\n- **Current best line:** gate + MRU demotion + probing overflow + probing room gate (0004 v002). The variant with a single bounded deferral (0005 v002) is a branch of it: better `put` p99, worse `get` tail and small-cache hit rate. Neither has a Winner yet. No variant matches baseline `get` p50.\n\n## Recurring failures\n\n- **Skipping or deferring cascades only moves their cost.** Unbounded deferral (0004) kept items in stash permanently and made `put` p99 much worse. Single deferral (0005) and the ungated overflow branch (0004) moved the cost to `get`. A change that skips promotions only works if items keep flowing out of stash.\n- **The `get` p50 gap has not been found on the common hit path.** Lookup reordering with stash last (0003, worse because stash hits are a large share of hits), scan direction (0005), removing the first-hit MRU move (0002) and skipping the near-MRU reorder (0006) all left p50 flat or worse.\n- **Weak signals for the gate.** A liveness signal from a single normal-tier stay (0005) lost hit rate everywhere. A gate on how full the stash bucket is (0006) never fires, because stash is nearly always full at steady state.\n- **The ungated MRU-demotion line is not cheaper.** It reaches the same slower `get` band as the gated line (0003), and adding overflow to it costs more `get` p99 than it saves on `put` (0004). Drop this line.\n- **Small p99 changes are unreliable.** Cost-only variants with identical policy made a p99 worse on paths they don't touch (0005, 0006). The cause is noise or codegen/inlining side effects. Don't decide anything from a small tail change without a rerun.\n- **Build infrastructure:** Maven/Gradle 429s killed variants in 0001. Treat build failures as untested ideas, not as negative results.\n\n## Open questions\n\n- **Stash probe cost:** the only remaining suspect for `get` p50 is the stash probe on every normal-tier hit. Possible fixes are a cheaper stash membership check or a skip filter for keys known not to be in stash. Moving stash to the end of the lookup has already been tried and was worse.\n- **Fused stash-promotion rotate (0006 v003):** it improved `get` p99 but made `put` p99 worse. Does keeping it to the third-hit path only, or marking it non-inline, keep the `get` gain without the `put` cost?\n- **Better deferral signal than fill level:** for example, whether the stash slot that would be evicted holds an item still waiting for its second hit. The goal is to keep 0005 v002's `put` p99 gain without its small-cache hit-rate loss.\n- **Size-dependent trade-off:** longer stash residency helps the large cache and hurts the small one. Could the policy adapt to cache size?\n- **Is `get` p50 parity with baseline achievable at all** while stash hits stay a large share of hits, or is part of the gap simply the price of the hit-rate gains?\n\n## Per-policy notes\n\n- **Dash (the only policy studied so far):**\n  - The stash-hit promotion chain (remove → promote → demote cascade) drives `get` latency. It scales with how many stash hits there are, not with the gate itself.\n  - `put` p99 follows stash pressure and how many cascades `put` has to absorb.\n  - Items placed in probing pay a lookup penalty: they are found last and can be evicted by a neighbour's promotion.\n  - MRU demotion for every demoted item is settled policy. Keep the two-hit gate.\n"
}
```

### 23:03:44 thinking



### 23:03:44 chat

I wrote `cross-cycle-insights.md` from the cycles 0001–0006 ledger. It has the four required sections in order and no numbers. No cycle has produced a Winner yet, and nothing matches baseline `get` p50.

- **Recurring wins:** Changes that keep proven items in the cache raised hit rate in every cycle: the two-hit gate, MRU demotion, probing overflow and the probing room gate. hm_0 at the large size is the cell that consistently shows retention effects. Probing overflow cut cascades without just moving their cost elsewhere. The current best line is 0004 v002, with the bounded-deferral variant 0005 v002 as a branch of it.
- **Recurring failures:**
  - Skipping or deferring cascades just moved their cost to a different path.
  - Every lever tried on the common `get` hit path left p50 flat or worse: lookup order, scan direction and the MRU move.
  - Two signals failed. A liveness check from a single stay in the normal tier lost hit rate, and the stash-fill gate never triggers.
  - The ungated MRU-demotion line is not a cheaper route.
  - Small p99 changes need a rerun before anyone acts on them.
  - The build failures in 0001 leave those ideas untested rather than disproven.
- **Open questions:**
  - Can the stash probe be made cheaper? It is the last suspect for the `get` p50 gap.
  - Does the fused rotate keep its `get` p99 gain if limited to the third-hit path or kept from being inlined?
  - Is there a better signal for deferring than how full the stash is?
  - Could the policy adapt to cache size, since longer stash stays help the large cache and hurt the small one?
  - Is matching baseline p50 possible at all while stash hits stay a large share of hits?
- **Per-policy notes:** Dash is the only policy in the ledger, so this section has one entry. It covers what drives `get` and `put` latency, the cost of finding items placed in the probing bucket, and the parts now treated as settled: MRU demotion for every demoted item, and keeping the two-hit gate.


---

- finished: 2026-11-05T23:03:45+00:00
- duration: 26.85s
- status: ok
- input_tokens: 4
- output_tokens: 2363
- cache_read_tokens: 39710
- cache_write_tokens: 11399
- cost_usd: 0.147098
